use std::arch::x86_64::*;

/* Reference: https://lemire.me/blog/2017/01/20/how-quickly-can-you-remove-spaces-from-a-string/ */
pub struct SimdString {
    /* FIXME: Would it be better to own the slice instead of the String? */
    s: String,
}

impl SimdString {
    pub fn new(s: String) -> Self {
        Self { s }
    }

    pub fn trim_start_matches<'a>(&'a self, c: char) -> &'a str {
        let len = self.s.len();
        // TODO: support the string with length doesn't align to 8
        if (len < 8) || (len & 0x7 != 0) {
            panic!();
        }

        let src: &[u8] = self.s.as_bytes();
        let mut pos: usize = 0;
        let mut i = 0;
        unsafe {
            while i < len {
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
                i += 16;
            }

            self.s.get_unchecked(pos..len)
        }
    }

    pub fn trim_start(&self) -> &str {
        self.trim_start_matches(' ')
    }
}

impl From<&str> for SimdString {
    #[inline]
    fn from(s: &str) -> SimdString {
        SimdString::new(s.to_owned())
    }
}

pub trait ToSimdString {
    fn to_simd_string(&self) -> SimdString;
}

impl ToSimdString for str {
    #[inline]
    fn to_simd_string(&self) -> SimdString {
        SimdString::from(self)
    }
}
