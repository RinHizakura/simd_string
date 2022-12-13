pub mod simd_string;

#[cfg(test)]
mod tests {
    use crate::simd_string::*;

    #[test]
    fn test_trim_start() {
        let test_strs = vec!["                  123456", "12345678"];
        for test_str in test_strs {
            let s1 = test_str.to_string();
            let s2 = test_str.to_simd_string();
            let s1 = s1.trim_start().to_string();
            let s2 = s2.trim_start().to_string();
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_trim_match() {
        let test_chs = vec!['@', '+'];
        let test_strs = vec!["@@@@@@@@@@@@@@@@@@123456", "++123456"];
        for pair in test_strs.into_iter().zip(test_chs.into_iter()) {
            let (test_str, test_ch) = pair;
            let s1 = test_str.to_string();
            let s2 = test_str.to_simd_string();
            let s1 = s1.trim_start_matches(test_ch).to_string();
            let s2 = s2.trim_start_matches(test_ch).to_string();
            assert_eq!(s1, s2);
        }
    }
}
