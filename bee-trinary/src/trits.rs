//! Converter functions that convert to various datatypes to Trits.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::{
    constants::SIG_MSG_FRG_SIZE_TRITS,
    constants::TRANSACTION_SIZE_TRITS,
    luts::ASCII_CODE_TO_TRITS,
    luts::ASCII_CODE_TO_TRYTE_CODE,
    luts::TRYTE_CODE_TO_TRITS,
    types::Trit,
    types::Tryte,
};

macro_rules! from_bytes_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts fixed-sized slices of bytes to trits.
        pub fn $func_name(bytes: &[u8]) -> [Trit; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                assert_eq!(0, bytes.len() % 2);
                assert_eq!($length, bytes.len() / 2 * 9);
            }

            let mut trits = [0_i8; $length];

            for i in 0..$length / 9 {
                let b0 = bytes[2 * i] as usize;
                let b1 = bytes[2 * i + 1] as usize;

                let offset = i * 9;

                trits[offset..offset + 3].copy_from_slice(&TRYTE_CODE_TO_TRITS[b0 / 8][..]);
                trits[(offset + 3)..(offset + 6)].copy_from_slice(&TRYTE_CODE_TO_TRITS[b1 / 8]);
                trits[(offset + 6)..(offset + 9)]
                    .copy_from_slice(&TRYTE_CODE_TO_TRITS[b0 % 8 + 8 * (b1 % 8)]);
            }

            trits
        }
    };
}

from_bytes_conv!(from_bytes_all, TRANSACTION_SIZE_TRITS);
from_bytes_conv!(from_bytes_sig, SIG_MSG_FRG_SIZE_TRITS);
from_bytes_conv!(from_bytes_54, 243);
from_bytes_conv!(from_bytes_18, 81);
from_bytes_conv!(from_bytes_6, 27);

/// Converts arbitrary slices of bytes to trits.
pub fn from_bytes(bytes: &[u8]) -> Vec<Trit> {
    #[cfg(not(feature = "no_checks"))]
    {
        assert_eq!(0, bytes.len() % 2);
    }

    let mut trits = vec![0_i8; bytes.len() / 2 * 9];

    for i in 0..(trits.len() / 9) {
        let pos = 2 * i;

        let b0 = bytes[pos] as usize;
        let b1 = bytes[pos + 1] as usize;

        let offset = i * 9;
        trits[offset..offset + 3].copy_from_slice(&TRYTE_CODE_TO_TRITS[b0 / 8][..]);
        trits[(offset + 3)..(offset + 6)].copy_from_slice(&TRYTE_CODE_TO_TRITS[b1 / 8]);
        trits[(offset + 6)..(offset + 9)]
            .copy_from_slice(&TRYTE_CODE_TO_TRITS[b0 % 8 + 8 * (b1 % 8)]);
    }

    trits
}

macro_rules! from_tryte_str_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts fixed-length slices of tryte strings to trits.
        pub fn $func_name(tryte_str: &str) -> [Trit; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                assert!(crate::util::is_tryte_str(tryte_str));
                assert_eq!($length, tryte_str.len() * 3);
            }

            let trytes = tryte_str.as_bytes();
            let mut trits = [0i8; $length];

            trytes.iter().enumerate().for_each(|(i, c)| {
                trits[(i * 3)..(i * 3) + 3]
                    .copy_from_slice(&TRYTE_CODE_TO_TRITS[ASCII_CODE_TO_TRYTE_CODE[c]][..]);
            });

            trits
        }
    };
}

from_tryte_str_conv!(from_tryte_str_all, TRANSACTION_SIZE_TRITS);
from_tryte_str_conv!(from_tryte_str_sig, SIG_MSG_FRG_SIZE_TRITS);
from_tryte_str_conv!(from_tryte_str_81, 243);
from_tryte_str_conv!(from_tryte_str_27, 81);
from_tryte_str_conv!(from_tryte_str_9, 27);

/// Converts arbitrary slices of tryte strings to trits.
pub fn from_tryte_str(tryte_str: &str) -> Vec<Trit> {
    #[cfg(not(feature = "no_checks"))]
    {
        assert!(crate::util::is_tryte_str(tryte_str));
    }
    let bytes = tryte_str.as_bytes();

    let mut trits = vec![0_i8; tryte_str.len() * 3];

    bytes.iter().enumerate().for_each(|(i, c)| {
        trits[(i * 3)..(i * 3) + 3]
            .copy_from_slice(&TRYTE_CODE_TO_TRITS[ASCII_CODE_TO_TRYTE_CODE[c]][..]);
    });

    trits
}

macro_rules! from_trytes_conv {
    ($func_name:ident, $length:expr) => {
        /// Converts fixed-length slices of trytes to trits.
        pub fn $func_name(trytes: &[Tryte]) -> [Trit; $length] {
            #[cfg(not(feature = "no_checks"))]
            {
                assert!(crate::util::is_trytes(trytes));
                assert_eq!($length, trytes.len() * 3);
            }

            let mut trits = [0_i8; $length];
            trytes.iter().enumerate().for_each(|(i, t)| {
                trits[(i * 3)..(i * 3 + 3)].copy_from_slice(&ASCII_CODE_TO_TRITS[t][..]);
            });

            trits
        }
    };
}

from_trytes_conv!(from_trytes_all, TRANSACTION_SIZE_TRITS);
from_trytes_conv!(from_trytes_sig, SIG_MSG_FRG_SIZE_TRITS);
from_trytes_conv!(from_trytes_81, 243);
from_trytes_conv!(from_trytes_27, 81);
from_trytes_conv!(from_trytes_9, 27);

/// Converts arbitrary slices of trytes to trits.
pub fn from_trytes(trytes: &[Tryte]) -> Vec<Trit> {
    let mut trits = vec![0_i8; trytes.len() * 3];

    trytes.iter().enumerate().for_each(|(i, t)| {
        trits[(i * 3)..(i * 3 + 3)].copy_from_slice(&ASCII_CODE_TO_TRITS[t][..]);
    });

    trits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_tryte_string_test() {
        //
        //println!("{:?}", from_tryte_str("AAA"));
        assert_eq!(&[1, 0, 0, 1, 0, 0, 1, 0, 0], &from_tryte_str("AAA")[..]);

        //println!("{:?}", from_tryte_str("SEG"));
        assert_eq!(&[1, 0, -1, -1, -1, 1, 1, -1, 1], &from_tryte_str("SEG")[..]);
    }

}
