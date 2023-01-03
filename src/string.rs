use crate::align_down;
use crate::from::FromSimdString;
use crate::pattern::SimdPattern;
use std::arch::x86_64::*;

pub trait SimdString {
    /* FIXME: Is it possible to change the type to generic type with trait
     * Pattern implemented? */
    fn simd_trim_start_matches_ch<'a>(&'a self, c: char) -> &'a str;
    fn simd_trim_start_matches_str<'a>(&'a self, s: &str) -> &'a str;

    fn simd_find_ch(&self, c: char) -> Option<usize>;
    fn simd_find_str<'a>(&'a self, s: &str) -> Option<usize>;

    fn simd_trim_start(&self) -> &str;
    fn simd_parse<F: FromSimdString>(&self) -> Result<F, F::Err>;
}

fn simd_cmp_mask<P: SimdPattern>(src: &[u8], offset: usize, pat: &P) -> i32 {
    let comp = pat.into_comparator(offset);
    let result;
    unsafe {
        let x = _mm_loadu_si128(src.as_ptr().offset(offset as isize) as *const _);

        /* Compare each char with comparator, set bit 1 on the corresponding
         * position if equal. */
        let xcomp = _mm_cmpeq_epi8(x, comp);
        result = _mm_movemask_epi8(xcomp);
    }
    result
}

fn simd_trim_start_matches<'a, P: SimdPattern>(s: &'a str, pat: P) -> Option<&'a str> {
    let len = s.len();
    let len0 = align_down!(len, 16);

    let src: &[u8] = s.as_bytes();
    let mut pos: usize = 0;
    for i in (0..len0).step_by(16) {
        let result = simd_cmp_mask(src, i, &pat);

        /* The mask 0xffff means that all of the 16 char we have
         * checked in this turn are the same as comparator. */
        if result != 0xffff {
            pos += (32 - result.leading_zeros()) as usize;
            unsafe {
                return Some(s.get_unchecked(pos..len));
            }
        }
        pos += 16;
    }

    None
}

impl SimdString for str {
    fn simd_trim_start_matches_ch<'a>(&'a self, c: char) -> &'a str {
        if let Some(s) = simd_trim_start_matches(self, c) {
            s
        } else {
            /* Fallback to default trim_start if:
             * 1. The character which should be trimmed lands in the remaning bytes
             * 2. The length of string just less than 16
             */
            self.trim_start_matches(c)
        }
    }

    fn simd_trim_start_matches_str<'a>(&'a self, s: &str) -> &'a str {
        if let Some(s) = simd_trim_start_matches(self, s) {
            s
        } else {
            self.trim_start_matches(s)
        }
    }

    fn simd_find_ch<'a>(&'a self, c: char) -> Option<usize> {
        let len = self.len();
        let len0 = align_down!(len, 16);

        let src: &[u8] = self.as_bytes();
        for i in (0..len0).step_by(16) {
            let result = simd_cmp_mask(src, i, &c);

            if result != 0 {
                return Some(i + result.trailing_zeros() as usize);
            }
        }

        if let Some(n) = self[len0..len].find(c) {
            return Some(len0 + n);
        }
        None
    }

    fn simd_find_str<'a>(&'a self, s: &str) -> Option<usize> {
        let pat_len = s.len();
        let s_slice: &[u8] = s.as_ref();

        let len = self.len();

        let src: &[u8] = self.as_bytes();
        let mut i = 0;
        while i + pat_len - 1 + 16 <= len {
            /* TODO:
             * 1. We can reduce an _mm_movemask_epi8 by adding an _mm256_and_si256
             * if this run faster
             * 2. better optimize this to deal with worst case */
            let mask1 = simd_cmp_mask(src, i, &(s_slice[0] as char));
            let mask2 = simd_cmp_mask(src, i + pat_len - 1, &(s_slice[pat_len - 1] as char));

            let mut mask = mask1 & mask2;
            while mask != 0 {
                let pos = mask.trailing_zeros() as usize;

                /* FIXME: It could be optimized with SIMD? */
                if src[i + pos..i + pos + pat_len] == *s_slice {
                    return Some(i + pos);
                }

                mask ^= 1 << pos;
            }

            i += 16;
        }

        if let Some(n) = self[i..len].find(s) {
            return Some(i + n);
        }
        None
    }

    fn simd_trim_start(&self) -> &str {
        self.trim_start_matches(' ')
    }

    fn simd_parse<F: FromSimdString>(&self) -> Result<F, F::Err> {
        FromSimdString::from_str(self)
    }
}
