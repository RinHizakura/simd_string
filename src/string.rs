use crate::tofrom_trait::FromSimdString;
use std::arch::x86_64::*;

/* Reference:
 * - https://lemire.me/blog/2017/01/20/how-quickly-can-you-remove-spaces-from-a-string/
 * - https://vgatherps.github.io/2022-11-28-dec/
 * */
pub struct SimdString {
    /* FIXME: Would it be better to own the slice instead of the String? */
    pub s: String,
}

impl SimdString {
    pub fn new(s: String) -> Self {
        Self { s }
    }

    pub fn trim_start_matches<'a>(&'a self, c: char) -> &'a str {
        let len = self.s.len();
        // TODO: support the string with length doesn't align to 16
        if (len < 16) || (len & 0xf != 0) {
            panic!("len = {} is not supported now", len);
        }

        let src: &[u8] = self.s.as_bytes();
        let mut pos: usize = 0;
        unsafe {
            for i in (0..len).step_by(16) {
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
                    break;
                }
                pos += 16;
            }

            self.s.get_unchecked(pos..len)
        }
    }

    pub fn trim_start(&self) -> &str {
        self.trim_start_matches(' ')
    }

    pub fn parse<F: FromSimdString>(&self) -> Result<F, F::Err> {
        FromSimdString::from_str(self)
    }
}

impl From<&str> for SimdString {
    #[inline]
    fn from(s: &str) -> SimdString {
        SimdString::new(s.to_owned())
    }
}
