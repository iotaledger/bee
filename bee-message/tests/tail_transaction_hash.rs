// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

const HASH_TRYTES: &str = "TZTXLMTAURX9DYQICXZEUMCDBPNXVOHNBBZDSSVCNCTWKSMUALAYPMHUCKGOGSTBUHSQIMSY9HQEP9AXJ";
const HASH_BYTES: [u8; 49] = [
    222, 235, 107, 67, 2, 173, 253, 93, 165, 90, 166, 45, 102, 91, 19, 137, 71, 146, 156, 180, 248, 31, 56, 25, 68,
    154, 98, 100, 64, 108, 203, 48, 76, 75, 114, 150, 34, 153, 203, 35, 225, 120, 194, 175, 169, 207, 80, 229, 10,
];

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", TailTransactionHash::new(HASH_BYTES).unwrap()),
        "TailTransactionHash(TZTXLMTAURX9DYQICXZEUMCDBPNXVOHNBBZDSSVCNCTWKSMUALAYPMHUCKGOGSTBUHSQIMSY9HQEP9AXJ)"
    );
}

#[test]
fn new_valid() {
    assert!(TailTransactionHash::new(HASH_BYTES).is_ok());
}

#[test]
fn try_from_valid() {
    assert!(TailTransactionHash::try_from(HASH_BYTES).is_ok());
}

#[test]
fn new_invalid() {
    assert!(matches!(
        TailTransactionHash::new([0x7a; TAIL_TRANSACTION_HASH_LEN]),
        Err(Error::InvalidTailTransactionHash)
    ));
}

#[test]
fn as_ref_valid() {
    assert_eq!(TailTransactionHash::try_from(HASH_BYTES).unwrap().as_ref(), HASH_BYTES);
}

#[test]
fn to_str_valid() {
    assert_eq!(
        TailTransactionHash::try_from(HASH_BYTES).unwrap().to_string(),
        HASH_TRYTES
    );
}

#[test]
fn packed_len() {
    let tth = TailTransactionHash::try_from(HASH_BYTES).unwrap();

    assert_eq!(tth.packed_len(), TAIL_TRANSACTION_HASH_LEN);
    assert_eq!(tth.pack_new().len(), TAIL_TRANSACTION_HASH_LEN);
}

#[test]
fn pack_unpack_valid() {
    let tth_1 = TailTransactionHash::try_from(HASH_BYTES).unwrap();
    let tth_1_bytes = tth_1.pack_new();
    let tth_2 = TailTransactionHash::unpack(&mut tth_1_bytes.as_slice()).unwrap();

    assert_eq!(tth_1, tth_2);
}

#[test]
fn pack_unpack_invalid() {
    let bytes = vec![
        0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a,
        0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a,
        0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a, 0x7a,
    ];

    assert!(matches!(
        TailTransactionHash::unpack(&mut bytes.as_slice()),
        Err(Error::InvalidTailTransactionHash)
    ));
}
