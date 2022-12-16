use crate::align_down;
use crate::from::FromSimdString;
use std::arch::x86_64::*;

/* Reference:
 * - https://lemire.me/blog/2017/01/20/how-quickly-can-you-remove-spaces-from-a-string/
 * - https://vgatherps.github.io/2022-11-28-dec/
 * */
pub trait SimdString {
    fn simd_trim_start_matches<'a>(&'a self, c: char) -> &'a str;
    fn simd_trim_start(&self) -> &str;
    fn simd_parse<F: FromSimdString>(&self) -> Result<F, F::Err>;
}

impl SimdString for str {
    fn simd_trim_start_matches<'a>(&'a self, c: char) -> &'a str {
        let len = self.len();
        let len0 = align_down!(len, 16);

        let src: &[u8] = self.as_bytes();
        let mut pos: usize = 0;
        unsafe {
            for i in (0..len0).step_by(16) {
                let spaces = _mm_set1_epi8(c as u8 as i8);
                let x = _mm_loadu_si128(src.as_ptr().offset(i as isize) as *const _);

                /* Compare each char with space, set bit 1 on the corresponding
                 * position if equal. */
                let xspaces = _mm_cmpeq_epi8(x, spaces);
                let result = _mm_movemask_epi8(xspaces);

                /* The mask 0xffff means that all of the 16 char we have
                 * checked in this turn are space. */
                if result != 0xffff {
                    pos += (32 - result.leading_zeros()) as usize;
                    return self.get_unchecked(pos..len);
                }
                pos += 16;
            }

            /* Fallback to default trim_start if:
             * 1. The character which should be trimmed lands in the remaning bytes
             * 2. The length of string just less than 16
             */
            self.trim_start_matches(c)
        }
    }

    fn simd_trim_start(&self) -> &str {
        self.trim_start_matches(' ')
    }

    fn simd_parse<F: FromSimdString>(&self) -> Result<F, F::Err> {
        FromSimdString::from_str(self)
    }
}
