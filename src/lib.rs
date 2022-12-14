mod string;
mod tofrom_trait;

#[macro_use]
pub mod macros;

pub use crate::string::*;
pub use crate::tofrom_trait::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_start() {
        let leading_space = [0, 1, 15, 16, 17, 23, 24, 25, 31, 32, 33, 100];

        for l in leading_space {
            let s = String::from_utf8(vec![b' '; l]).unwrap() + "HAPPY";

            let s1 = s.to_string();
            let s2 = s.to_simd_string();
            let s1 = s1.trim_start().to_string();
            let s2 = s2.trim_start().to_string();
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
                let s1 = s.to_string();
                let s2 = s.to_simd_string();
                let s1 = s1.trim_start_matches(ch).to_string();
                let s2 = s2.trim_start_matches(ch).to_string();
                assert_eq!(s1, s2);
            }
        }
    }

    #[test]
    fn test_parse_u64() {
        let test_strs = vec![
            "1234",
            "1234567812345678",
            "9876543200000000",
            "19876543200000000",
            "1987654320000000@",
            "@1987654320000000",
        ];
        for s in test_strs {
            let s1 = s.to_string();
            let s2 = s.to_simd_string();
            let s1 = s1.parse::<u64>();
            let s2 = s2.parse::<u64>();
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_parse_i64() {
        let test_strs = vec![
            "1234",
            "1234567812345678",
            "9876543200000000",
            "19876543200000000",
            "1987654320000000@",
            "-1987654320000000",
        ];
        for s in test_strs {
            let s1 = s.to_string();
            let s2 = s.to_simd_string();
            let s1 = s1.parse::<i64>();
            let s2 = s2.parse::<i64>();
            assert_eq!(s1, s2);
        }
    }
}
