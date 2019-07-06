//! Converter functions that convert trits/trytes to numbers.
//!
//! Note: Currently 11 trytes / 33 trits are sufficient to represent the total IOTA supply.
//! of (3^33-1)/2 = 2,779530283×10^15 since an `i64` can even hold (2^64-1)/2 =
//! 9,223372037×10^18 positive values. However, in the future it is likely that with increased value
//! of each single token fractional iotas will have to be introduced. In that case it is necessary
//! to represent (3^81-1)/2 = 2,217132441×10^38 different positive values, which couldn't even be
//! represented by an i128.

use crate::{
    luts::ASCII_CODE_SEQ_TO_NUM,
    luts::ASCII_CODE_TO_TRITS,
    types::Sign,
    types::Trit,
    types::Tryte,
    types::I129,
};

/// Converts up to 11 trytes to an `i64` by using only a LUT and addition.
///
/// This function panics if the slice is bigger then 11 trytes.
pub fn from_trytes_max11(trytes: &[Tryte]) -> i64 {
    #[cfg(not(feature = "no_checks"))]
    {
        assert!(crate::util::is_trytes(&trytes));
        assert!(trytes.len() <= crate::constants::TRYTE_LENGTH_FOR_MAX_TOKEN_SUPPLY);
    }

    let mut number = 0;
    trytes.iter().enumerate().for_each(|(i, &t)| {
        number += ASCII_CODE_SEQ_TO_NUM[i][t as usize - 57];
    });

    number
}

/// Converts up to 13 trytes to an `i64`.
///
/// This function panics if the slice is bigger then 13 trytes.
pub fn from_trytes_max13(trytes: &[Tryte]) -> i64 {
    #[cfg(not(feature = "no_checks"))]
    {
        assert!(crate::util::is_trytes(&trytes));
        assert!(trytes.len() <= crate::constants::TRYTE_LENGTH_FOR_MAX_I64);
    }

    let mut number = 0;
    trytes.iter().rev().for_each(|tryte| {
        let trits = ASCII_CODE_TO_TRITS[tryte];
        for n in (0..trits.len()).rev() {
            number = number * 3 + i64::from(trits[n]);
        }
    });

    number
}

/// Converts 27 trytes to an `I129`.
///
/// This function panics if the slice is bigger then 27 trytes.
pub fn from_trytes_max27(trytes: &[Tryte]) -> I129 {
    #[cfg(not(feature = "no_checks"))]
    {
        assert!(crate::util::is_trytes(&trytes));
        assert!(trytes.len() <= 27);
    }

    let mut sign = None;
    let mut number = 0_u128;

    trytes.iter().rev().for_each(|tryte| {
        let trits = ASCII_CODE_TO_TRITS[tryte];
        for n in (0..3).rev() {
            if sign.is_none() {
                sign = match trits[n] {
                    -1 => Some(Sign::Neg),
                    1 => Some(Sign::Pos),
                    _ => None,
                }
            }
            number *= 3;
            match trits[n] {
                -1 => match sign {
                    Some(Sign::Pos) => number -= 1,
                    Some(Sign::Neg) => number += 1,
                    _ => (),
                },
                1 => match sign {
                    Some(Sign::Pos) => number += 1,
                    Some(Sign::Neg) => number -= 1,
                    _ => (),
                },
                _ => (),
            }
        }
    });

    I129(sign.unwrap_or(Sign::Pos), number)
}

/// Converts a slice of trits to an `i64`.
///
/// This function panics if the slice is bigger then 39 trits.
pub fn from_trits(trits: &[Trit]) -> i64 {
    #[cfg(not(feature = "no_checks"))]
    {
        assert!(crate::util::is_trits(&trits));
        assert!(trits.len() <= 39);
    }

    let mut number = 0;

    for n in (0..trits.len()).rev() {
        number = number * 3 + i64::from(trits[n]);
    }

    number
}

#[cfg(test)]
mod tests {
    use super::super::trytes;
    use super::*;
    use rand::*;

    #[test]
    fn test_from_trytes_max11() {
        let number = from_trytes_max11(&['9' as u8]);
        assert_eq!(0, number);

        let number = from_trytes_max11(&['A' as u8]);
        assert_eq!(1, number);

        let number = from_trytes_max11(&['M' as u8]);
        assert_eq!(13, number);

        let number = from_trytes_max11(&['A' as u8, '9' as u8]);
        assert_eq!(1, number);

        let number = from_trytes_max11(&['9' as u8, 'A' as u8]);
        assert_eq!(27, number);

        let number = from_trytes_max11(&['N' as u8]);
        assert_eq!(-13, number);
    }

    #[test]
    fn test_from_trytes_max13() {
        let number = from_trytes_max13(&['9' as u8]);
        assert_eq!(0, number);

        let number = from_trytes_max13(&['A' as u8]);
        assert_eq!(1, number);

        let number = from_trytes_max13(&['M' as u8]);
        assert_eq!(13, number);

        let number = from_trytes_max13(&['A' as u8, '9' as u8]);
        assert_eq!(1, number);

        let number = from_trytes_max13(&['9' as u8, 'A' as u8]);
        assert_eq!(27, number);

        let number = from_trytes_max11(&['N' as u8]);
        assert_eq!(-13, number);
    }

    #[test]
    fn test_from_trytes_max11_inverse() {
        for _ in 0..1000 {
            let a = i64::from(thread_rng().next_u32());
            let b = i64::from(thread_rng().next_u32());
            let c = a - b;
            assert_eq!(c, from_trytes_max11(&trytes::from_num_i64_to_11(c)));
        }
    }

    #[test]
    fn test_from_trytes_max27() {
        let number = from_trytes_max27("9".as_bytes());
        assert_eq!(I129(Sign::Pos, 0), number);

        let number = from_trytes_max27("A".as_bytes());
        assert_eq!(I129(Sign::Pos, 1), number);

        let number = from_trytes_max27("M".as_bytes());
        assert_eq!(I129(Sign::Pos, 13), number);

        let number = from_trytes_max27("A9".as_bytes());
        assert_eq!(I129(Sign::Pos, 1), number);

        let number = from_trytes_max27("9A".as_bytes());
        assert_eq!(I129(Sign::Pos, 27), number);

        let number = from_trytes_max27("N".as_bytes());
        assert_eq!(I129(Sign::Neg, 13), number);

        let number = from_trytes_max27("NN".as_bytes());
        assert_eq!(I129(Sign::Neg, 364), number);

        let number = from_trytes_max27("MMMMMMMMMMMMMMMMMMMMMMMMMMM".as_bytes());
        assert_eq!(I129(Sign::Pos, 221713244121518884974124815309574946401), number);

        let number = from_trytes_max27("NNNNNNNNNNNNNNNNNNNNNNNNNNN".as_bytes());
        assert_eq!(I129(Sign::Neg, 221713244121518884974124815309574946401), number);
    }

    #[test]
    fn test_from_trits() {
        let number = from_trits(&[0, 0, 0]);
        assert_eq!(0, number);

        let number = from_trits(&[1, 1, 1]);
        assert_eq!(13, number);

        let number = from_trits(&[-1, -1, -1]);
        assert_eq!(-13, number);

        let number = from_trits(&[-1, -1, -1, 1]);
        assert_eq!(14, number);
    }

}
