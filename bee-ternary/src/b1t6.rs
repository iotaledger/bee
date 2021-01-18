// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Btrit, RawEncoding, RawEncodingBuf, TritBuf, Trits, Tryte};

use std::convert::TryFrom;

const TRITS_PER_BYTE: usize = 2;
const TRITS_PER_TRYTE: usize = 3;

/// Encode a series of trits into bytes.
pub fn decode(src: &Trits) -> Vec<u8> {
    assert!(src.len() % TRITS_PER_BYTE == 0);
    src.iter_trytes()
        .zip(src[TRITS_PER_TRYTE..].iter_trytes())
        .map(|(a, b)| decode_group(a as i8, b as i8).unwrap() as u8)
        .collect()
}

fn decode_group(t1: i8, t2: i8) -> Option<i8> {
    i8::try_from(t1 as isize + t2 as isize * 27).ok()
}

/// Decode a series of bytes into trits.
pub fn encode<T: RawEncodingBuf>(bytes: &[u8]) -> TritBuf<T>
where
    T::Slice: RawEncoding<Trit = Btrit>,
{
    let mut trits = TritBuf::new();

    for byte in bytes {
        let (t1, t2) = encode_group(*byte);
        [t1, t2]
            .iter()
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
