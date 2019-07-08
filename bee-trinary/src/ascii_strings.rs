//! Converter functions to convert trits/trytes to ASCII text.

#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::{
    constants::MAX_TRYTE_TRIPLET_ABS,
    luts::TRYTE_CODE_TO_ASCII_CODE,
    numbers::from_trytes_max11,
    types::Tryte,
    util::unpad_right,
};

/// Converts trytes to an ASCII/UTF8 encoded string.
pub fn from_trytes(trytes: &[Tryte]) -> String {
    if trytes.is_empty() {
        return String::new();
    }

    #[cfg(not(feature = "no_checks"))]
    {
        assert!(crate::util::is_trytes(trytes));
        assert_eq!(0, trytes.len() % 3);
    }

    inner(trytes)
}

/// Converts a tryte string to an ASCII/UTF8 string.
pub fn from_tryte_str(tryte_str: &str) -> String {
    if tryte_str.is_empty() {
        return String::new();
    };

    #[cfg(not(feature = "no_checks"))]
    {
        assert!(crate::util::is_tryte_str(tryte_str));
    }

    // Remove 9s from the str
    // TODO: instead of removing first and then adding again we should be able to
    // customize unpadding function
    let tryte_string = unpad_right(tryte_str);

    let mut trytes = tryte_string.as_bytes().to_vec();

    // make it a multiple of 3
    for _ in 0..trytes.len() % 3 {
        trytes.push(TRYTE_CODE_TO_ASCII_CODE[0]);
    }

    inner(&trytes)
}

#[inline]
fn inner(trytes: &[Tryte]) -> String {
    let mut ascii_chars = vec![0; trytes.len() / 3 * 2];

    for i in 0..trytes.len() / 3 {
        let index =
            from_trytes_max11(&trytes[(i * 3)..(i * 3 + 3)]) + MAX_TRYTE_TRIPLET_ABS;

        ascii_chars[i * 2] = (index / 127) as u8;
        ascii_chars[i * 2 + 1] = (index % 127) as u8;
    }

    if ascii_chars[ascii_chars.len() - 1] == 0 {
        ascii_chars.remove(ascii_chars.len() - 1);
    }

    // Since we checked all chars previously 'unwrap' should never fail
    String::from_utf8(ascii_chars).unwrap()
}

#[cfg(test)]
mod from_tryte_str_tests {
    use super::*;

    #[test]
    fn test_from_tryte_str() {
        assert_eq!("Hello", from_tryte_str("YEZNMEQWF"));
    }

    #[test]
    fn test_from_tryte_str_inverse() {
        assert_eq!(
            "Hello, IOTA!",
            from_tryte_str(&crate::tryte_strings::from_ascii("Hello, IOTA!"))
        );
    }
}
