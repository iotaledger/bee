//! Utility functions.

#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::luts::TRYTE_CODE_TO_ASCII_CODE;

use crate::{types::Trit, types::Tryte};

/// Determines whether the specified tryte string consists of tryte characters only.
pub fn is_tryte_str(tryte_str: &str) -> bool {
    tryte_str.chars().find(|c| *c != '9' && (*c < 'A' || *c > 'Z')).is_none()
}

/// Determines whether the specified slice consists of tryte characters only.
pub fn is_trytes(trytes: &[Tryte]) -> bool {
    trytes.iter().find(|t| **t != 57 && (**t < 65 || **t > 90)).is_none()
}

/// Determines whether the specified slice consists of balanced trinary trits only.
pub fn is_trits(trits: &[Trit]) -> bool {
    trits.iter().find(|t| !(**t == -1 || **t == 0 || **t == 1)).is_none()
}

/// Creates a padded tryte string.
pub fn pad_right(tryte_str: &str, length: usize) -> String {
    if length <= tryte_str.len() {
        return String::from(tryte_str);
    };

    let mut chars = vec![TRYTE_CODE_TO_ASCII_CODE[0]; length];

    let trytes = tryte_str.as_bytes();
    chars[0..trytes.len()].copy_from_slice(&trytes[..]);

    String::from_utf8(chars).unwrap()
}

/// Unpads a tryte string.
pub fn unpad_right(tryte_string: &str) -> String {
    match tryte_string.rfind(|c| c != '9') {
        Some(index) => String::from(&tryte_string[0..=index]),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tryte_str() {
        let test_trytes = "ABCDEFGHIJKLMNOPQRSTUVWXYZ99999";
        assert_eq!(true, is_tryte_str(test_trytes));

        let test_trytes = "ABCDEfGHIJKLMNOPQRSTUVWXYZ99999";
        assert_eq!(false, is_tryte_str(test_trytes));

        let test_trytes = "ABCDEFGHIJKLMNOPQRSTUVWXYZ99998";
        assert_eq!(false, is_tryte_str(test_trytes));
    }

    #[test]
    fn test_is_trytes() {
        assert_eq!(true, is_trytes(&[57, 65, 77, 90]));
        assert_eq!(false, is_trytes(&[56, 65, 77, 90]));
    }

    #[test]
    fn test_tryte_str() {
        //
        assert_eq!(true, is_tryte_str("ABCD9999"));
        assert_eq!(false, is_tryte_str("ABCD8999"));
        assert_eq!(false, is_tryte_str("aBCD9999"));
    }

    #[test]
    fn test_is_trits() {
        assert_eq!(true, is_trits(&[0, -1, 1, -1]));
        assert_eq!(false, is_trits(&[2, -1, 1, -1]));
    }

    #[test]
    fn test_pad_right() {
        assert_eq!("ABCD9999", pad_right("ABCD", 8));
        assert_eq!("ABCD", pad_right("ABCD", 3));
        assert_eq!("9999ABCD99", pad_right("9999ABCD", 10));
        assert_eq!("99AB99CD99", pad_right("99AB99CD", 10));
    }

    #[test]
    fn test_unpad_right() {
        assert_eq!("ABCD", unpad_right("ABCD9999"));
        assert_eq!("ABCD", unpad_right("ABCD"));
        assert_eq!("9999ABCD", unpad_right("9999ABCD99"));
        assert_eq!("99AB99CD", unpad_right("99AB99CD99"));
    }

    #[test]
    fn test_pad_unpad() {
        assert_eq!("9A9B9C9D", unpad_right(&pad_right("9A9B9C9D", 100)));
    }
}
