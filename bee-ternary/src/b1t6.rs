// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Btrit, RawEncoding, RawEncodingBuf, TritBuf, Trits, Tryte};

use std::convert::TryFrom;

const TRITS_PER_BYTE: usize = 2;
const TRITS_PER_TRYTE: usize = 3;

/// An error that may be emitted when decoding a B1T6 trit slice.
#[derive(Debug)]
pub enum DecodeError {
    /// Two trits had an invalid B1T6 representation.
    InvalidTrytes([Tryte; 2]),
}

/// Decode a series of trits into bytes.
pub fn decode(src: &Trits) -> Result<Vec<u8>, DecodeError> {
    assert!(src.len() % TRITS_PER_BYTE == 0);
    src.iter_trytes()
        .zip(src[TRITS_PER_TRYTE..].iter_trytes())
        .map(|(a, b)| decode_group(a, b).ok_or(DecodeError::InvalidTrytes([a, b])))
        .collect()
}

fn decode_group(t1: Tryte, t2: Tryte) -> Option<u8> {
    Some(i8::try_from(t1 as isize + t2 as isize * 27).ok()? as u8)
}

/// Encode a series of bytes into trits.
pub fn encode<T: RawEncodingBuf>(bytes: &[u8]) -> TritBuf<T>
where
    T::Slice: RawEncoding<Trit = Btrit>,
{
    let mut trits = TritBuf::new();

    for byte in bytes {
        let (t1, t2) = encode_group(*byte);
        [t1, t2]
            .iter()
            // Unwrap is safe, `encode_group` is valid for all inputs
            .for_each(|b| trits.append(Tryte::try_from(*b).unwrap().as_trits()));
    }

    trits
}

fn encode_group(byte: u8) -> (i8, i8) {
    let v = (byte as i8) as i16 + (27 / 2) * 27 + 27 / 2;
    let quo = (v / 27) as i8;
    let rem = (v % 27) as i8;

    (rem + Tryte::MIN_VALUE as i8, quo + Tryte::MIN_VALUE as i8)
}
