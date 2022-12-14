use crate::string::SimdString;
use std::arch::x86_64::*;
use std::num::ParseIntError;

pub trait ToSimdString {
    fn to_simd_string(&self) -> SimdString;
}

impl ToSimdString for str {
    #[inline]
    fn to_simd_string(&self) -> SimdString {
        SimdString::from(self)
    }
}

pub trait FromSimdString: Sized {
    type Err;

    fn from_str(s: &SimdString) -> Result<Self, Self::Err>;
}

impl FromSimdString for u64 {
    type Err = ParseIntError;

    fn from_str(s: &SimdString) -> Result<Self, Self::Err> {
        // TODO: consider sign for the number
        let len = s.s.len();
        // TODO: support the string with length doesn't align to 16
        if (len < 16) || (len & 0xf != 0) {
            panic!("len = {} is not supported now", len);
        }

        let mut result = 0;
        let src: &[u8] = s.s.as_bytes();
        unsafe {
            for i in (0..len).step_by(16) {
                let x = _mm_loadu_si128(src.as_ptr().offset(i as isize) as *const _);

                /* Make '0'..'9' become 0..9, otherwise the char will become garbage
                 * which could be checked then. */
                let ch_zeros = _mm_set1_epi8('0' as i8);
                let x = _mm_sub_epi8(x, ch_zeros);

                /* For every packed u8 integers, check whether it is less equal to 9
                 *
                 * FIXME: It looks like that we don't have stable API like
                 * _mm_cmplt_epu8 or _mm_cmpgt_epu8 now. Instead, we complete
                 * the similar behavior by:
                 * 1. For each u8 number, doing max(number, 9)
                 * 2. If this is a valid number, all numbers should equal to 9 */
                let nines = _mm_set1_epi8(9);
                let max_of_nine = _mm_max_epu8(x, nines);
                let is_eq_nine = _mm_cmpeq_epi8(nines, max_of_nine);
                if _mm_test_all_ones(is_eq_nine) == 0 {
                    /* FIXME: Fallback to normal parse even if we know that it
                     * should be error. We do this because there's no way to create
                     * our own ParseIntError. */
                    return s.s.parse::<u64>();
                }

                /* Compute the final result with multiply-add */
                /* x 16x8 -> 8x16 */
                let mul_1_10 =
                    _mm_setr_epi8(10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1);
                let x = _mm_maddubs_epi16(x, mul_1_10);

                /* x 8x16 -> 4x32 */
                let mul_1_100 = _mm_setr_epi16(100, 1, 100, 1, 100, 1, 100, 1);
                let x = _mm_madd_epi16(x, mul_1_100);

                /* We don't have multiply add from 4x32 to 2x64. However, for each
                 * packed 32 bit interger, they won't be larger than 2^16
                 * (because 100 * (9 * 10 + 9) + (9 * 10 + 9) = 9999), so we can put
                 * 4x32 into 16x8 and use _mm_madd_epi16. */
                let x = _mm_packs_epi32(x, x);
                /* x 4x32 -> 2x128 */
                let mul_1_10000 = _mm_setr_epi16(10000, 1, 10000, 1, 10000, 1, 10000, 1);
                let x = _mm_madd_epi16(x, mul_1_10000);

                /* Obtain the lowest 64bit and convert to the final result */
                let as_u64 = _mm_cvtsi128_si64(x) as u64;
                result += (as_u64 >> 32) + 1_0000_0000 * (as_u64 & 0xffff_ffff);
                // TODO
                break;
            }

            return Ok(result);
        }
    }
}
