mod from;
mod string;

#[macro_use]
mod macros;

pub use crate::from::*;
pub use crate::string::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_start() {
        let leading_space = [0, 1, 15, 16, 17, 23, 24, 25, 31, 32, 33, 100];

        for l in leading_space {
            let s = String::from_utf8(vec![b' '; l]).unwrap() + "HAPPY";

            let s1 = s.trim_start().to_string();
            let s2 = s.simd_trim_start().to_string();
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_trim_match() {
        let test_chs = vec!['@', '+'];
        let leading_ch = [0, 1, 15, 16, 17, 23, 24, 25, 31, 32, 33, 100];

        for ch in test_chs {
            for l in leading_ch {
                let s = String::from_utf8(vec![ch as u8; l]).unwrap() + "HAPPY";
                let s1 = s.trim_start_matches(ch).to_string();
                let s2 = s.simd_trim_start_matches(ch).to_string();
                assert_eq!(s1, s2);
            }
        }
    }

    #[test]
    fn test_parse_u64() {
        let test_strs = vec![
            "1234",
            "11234567812345678",
            "19876543200000000",
            "19876543200000000",
            "18446744073709551614",
            "18446744073709551615", // u64 max
            "18446744073709551616",
            "1987654320000000@",
            "@1987654320000000",
        ];
        for s in test_strs {
            let s1 = s.parse::<u64>();
            let s2 = s.simd_parse::<u64>();
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_parse_i64() {
        let test_strs = vec![
            "1234",
            "11234567812345678",
            "19876543200000000",
            "19876543200000000",
            "18446744073709551615",
            "1987654320000000@",
            "-1987654320000000",
            "-9223372036854775807",
            "-9223372036854775808", // i64 min
            "-9223372036854775809",
        ];
        for s in test_strs {
            let s1 = s.parse::<i64>();
            let s2 = s.simd_parse::<i64>();
            assert_eq!(s1, s2);
        }
    }
}
