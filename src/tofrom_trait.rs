use crate::align_down;
use crate::string::SimdString;
use std::arch::x86_64::*;
use std::num::ParseIntError;
use std::ops::{Add, Mul, Sub};

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

fn from_simd_str_radix<T>(s: &SimdString, radix: u32) -> Result<T, ParseIntError>
where
    T: FromStrRadixHelper + std::str::FromStr<Err = ParseIntError>,
{
    //TODO
    assert_eq!(radix, 10);

    let len = s.s.len();
    /* Use default parse for short string, so some ParseIntError
     * can also be handled there directly.
     *
     * Note: The condition is not "len < 16" because the first character could be sign */
    if len < 17 {
        return s.s.parse::<T>();
    }

    let src: &[u8] = s.s.as_bytes();
    let is_signed_ty = T::from_u64(0) > T::MIN;
    let (is_positive, src, start) = match src[0] {
        b'+' => (true, &src[1..], 1),
        b'-' if is_signed_ty => (false, &src[1..], 1),
        _ => (true, src, 0),
    };

    let len0 = align_down!(len - start, 16);
    let len1 = len - start - len0;

    // use default parse for the remaining bytes
    let mut result1: T = T::from_u64(0);
    if len1 > 0 {
        result1 = s.s[start + len0..len].parse::<T>()?;
    }

    let mut result0 = T::from_u64(0);
    unsafe {
        for i in (0..len0 - start).step_by(16) {
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
                return s.s.parse::<T>();
            }

            /* Compute the final result with multiply-add */
            /* x 16x8 -> 8x16 */
            let mul_1_10 = _mm_setr_epi8(10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1);
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

            // TODO: handle possible overflow
            let r = (as_u64 >> 32) + 1_0000_0000 * (as_u64 & 0xffff_ffff);
            result0 = result0 + T::from_u64(r);
            // TODO: don't break
            break;
        }
    }

    // TODO: handle possible overflow
    let mut result = result0 * T::from_u64(10_u64.pow(len1 as u32)) + result1;
    if !is_positive {
        result = T::from_u64(u64::MAX) * result;
    }
    Ok(result)
}

/* Reference: https://doc.rust-lang.org/src/core/num/mod.rs.html*/
trait FromStrRadixHelper:
    PartialOrd + Copy + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
    const MIN: Self;
    fn from_u64(u: u64) -> Self;
}

macro_rules! from_str_radix_int_impl {
    ($($t:ty)*) => {$(
        impl FromSimdString for $t {
            type Err = ParseIntError;
            fn from_str(s: &SimdString) -> Result<Self, ParseIntError> {
                from_simd_str_radix(s, 10)
            }
        }
    )*}
}

from_str_radix_int_impl! { isize i8 i16 i32 i64 i128 usize u8 u16 u32 u64 u128 }

macro_rules! impl_helper_for {
    ($($t:ty)*) => ($(impl FromStrRadixHelper for $t {
        const MIN: Self = Self::MIN;
        #[inline]
        fn from_u64(u: u64) -> Self { u as Self }
    })*)
}
impl_helper_for! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }
