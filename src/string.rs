use crate::align_down;
use crate::from::FromSimdString;
use crate::pattern::SimdPattern;
use std::arch::x86_64::*;

/* Reference:
 * - https://lemire.me/blog/2017/01/20/how-quickly-can-you-remove-spaces-from-a-string/
 * - https://vgatherps.github.io/2022-11-28-dec/
 * */
pub trait SimdString {
    /* FIXME: Is it possible to change the type to generic type with trait
     * Pattern implemented? */
    fn simd_trim_start_matches_ch<'a>(&'a self, c: char) -> &'a str;
    fn simd_trim_start_matches_str<'a>(&'a self, s: &str) -> &'a str;

    fn simd_trim_start(&self) -> &str;
    fn simd_parse<F: FromSimdString>(&self) -> Result<F, F::Err>;
}

fn simd_trim_start_matches<'a, P: SimdPattern>(s: &'a str, pat: P) -> Option<&'a str> {
    let len = s.len();
    let len0 = align_down!(len, 16);

    let src: &[u8] = s.as_bytes();
    let mut pos: usize = 0;
    unsafe {
        for i in (0..len0).step_by(16) {
            let comp = pat.into_comparator(i);
            let x = _mm_loadu_si128(src.as_ptr().offset(i as isize) as *const _);

            /* Compare each char with space, set bit 1 on the corresponding
             * position if equal. */
            let xcomp = _mm_cmpeq_epi8(x, comp);
            let result = _mm_movemask_epi8(xcomp);

            /* The mask 0xffff means that all of the 16 char we have
             * checked in this turn are space. */
            if result != 0xffff {
                pos += (32 - result.leading_zeros()) as usize;
                return Some(s.get_unchecked(pos..len));
            }
            pos += 16;
        }
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

    fn simd_trim_start(&self) -> &str {
        self.trim_start_matches(' ')
    }

    fn simd_parse<F: FromSimdString>(&self) -> Result<F, F::Err> {
        FromSimdString::from_str(self)
    }
}
