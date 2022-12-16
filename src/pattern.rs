use std::arch::x86_64::*;

pub trait SimdPattern {
    fn into_comparator(&self, offset: usize) -> __m128i;
}

impl SimdPattern for char {
    fn into_comparator(&self, _offset: usize) -> __m128i {
        unsafe { _mm_set1_epi8(*self as u8 as i8) }
    }
}

impl SimdPattern for &str {
    fn into_comparator(&self, offset: usize) -> __m128i {
        /* FIXME: Will this cost a lot? */
        let len = self.len();
        let mut src = [0; 16];
        for i in 0..16 {
            src[i] = self.as_bytes()[(offset + i) % len];
        }
        unsafe { _mm_loadu_si128(src.as_ptr() as *const _) }
    }
}
