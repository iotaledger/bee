// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::{TritBuf, Trits, Tryte};

use std::convert::TryFrom;

const TRITS_PER_TRYTE: usize = 3;
const TRITS_PER_BYTE: usize = 6;

// TODO better err handling
// TODO complete set of test
// TODO more efficient impls

pub fn decode(src: &Trits) -> Vec<u8> {
    if src.len() % TRITS_PER_BYTE != 0 {
        // TODO do something
        panic!();
    }

    let mut bytes = Vec::with_capacity(src.len() / TRITS_PER_BYTE);

    for j in (0..src.len()).step_by(TRITS_PER_BYTE) {
        let t1 = i8::try_from(&src[j..j + TRITS_PER_TRYTE]).unwrap();
        let t2 = i8::try_from(&src[j + TRITS_PER_TRYTE..j + TRITS_PER_BYTE]).unwrap();
        let b = decode_group(t1, t2).unwrap();
        bytes.push(b as u8);
    }

    bytes
}

fn decode_group(t1: i8, t2: i8) -> Result<i8, ()> {
    let v = t1 as isize + t2 as isize * 27;

    i8::try_from(v).map_err(|_| ())
}

pub fn encode(bytes: &[u8]) -> TritBuf {
    let mut trits = TritBuf::new();

    for byte in bytes {
        let (t1, t2) = encode_group(*byte);
        // TODO unwrap
        Tryte::try_from(t1)
            .unwrap()
            .as_trits()
            .iter()
            .for_each(|t| trits.push(t));
        Tryte::try_from(t2)
            .unwrap()
            .as_trits()
            .iter()
            .for_each(|t| trits.push(t));
    }

    trits
}

fn encode_group(byte: u8) -> (i8, i8) {
    let v = (byte as i8) as i16 + (27 / 2) * 27 + 27 / 2;
    let quo = (v / 27) as i8;
    let rem = (v % 27) as i8;

    (rem + Tryte::MIN_VALUE as i8, quo + Tryte::MIN_VALUE as i8)
}
