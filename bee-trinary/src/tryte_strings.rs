//! Converter functions that convert trits/trytes and other datatypes to tryte strings.

#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::{trytes, types::Trit, types::Tryte};

/// Converts a slice of trits to a tryte string.
pub fn from_trits(trits: &[Trit]) -> String {
    String::from_utf8(trytes::from_trits(&trits)).unwrap()
}

#[cfg(test)]
#[test]
fn test_from_trits() {
    assert_eq!("A", from_trits(&[1, 0, 0]));
    assert_eq!("M", from_trits(&[1, 1, 1]));
    assert_eq!("9A", from_trits(&[0, 0, 0, 1, 0, 0]));
}

/// Converts a slice of trytes to a tryte string.
pub fn from_trytes(trytes: &[Tryte]) -> String {
    String::from_utf8(trytes.to_vec()).unwrap()
}

#[cfg(test)]
#[test]
fn test_from_trytes() {
    assert_eq!("A", from_trytes(&['A' as u8]));
    assert_eq!("M", from_trytes(&['M' as u8]));
    assert_eq!("9A", from_trytes(&['9' as u8, 'A' as u8]));
}

/// Converts an ASCII string to a tryte string.
pub fn from_ascii(ascii_str: &str) -> String {
    String::from_utf8(trytes::from_ascii(ascii_str)).unwrap()
}

#[cfg(test)]
#[test]
fn test_from_ascii() {
    assert_eq!("YEZNMEQWF", from_ascii("Hello"));
}
