// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_test::rand::receipt::rand_tail_transaction_hash;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn new_valid() {
    let tth = rand_tail_transaction_hash();
    let output = SignatureLockedSingleOutput::new(
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        42424242,
    )
    .unwrap();
    let mfe = MigratedFundsEntry::new(tth.clone(), output.clone()).unwrap();

    assert_eq!(mfe.tail_transaction_hash(), &tth);
    assert_eq!(*mfe.output(), output);
}

#[test]
fn new_invalid_amount() {
    assert!(matches!(
        MigratedFundsEntry::new(
            rand_tail_transaction_hash(),
            SignatureLockedSingleOutput::new(Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()), 42)
                .unwrap()
        ),
        Err(Error::InvalidMigratedFundsEntryAmount(42))
    ));
}

#[test]
fn packed_len() {
    let mge = MigratedFundsEntry::new(
        rand_tail_transaction_hash(),
        SignatureLockedSingleOutput::new(
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            42424242,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(mge.packed_len(), 49 + 1 + 32 + 8);
    assert_eq!(mge.pack_new().len(), 49 + 1 + 32 + 8);
}

#[test]
fn pack_unpack_valid() {
    let mfe_1 = MigratedFundsEntry::new(
        rand_tail_transaction_hash(),
        SignatureLockedSingleOutput::new(
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            42424242,
        )
        .unwrap(),
    )
    .unwrap();
    let mfe_2 = MigratedFundsEntry::unpack(&mut mfe_1.pack_new().as_slice()).unwrap();

    assert_eq!(mfe_1.tail_transaction_hash(), mfe_2.tail_transaction_hash());
    assert_eq!(*mfe_1.output(), *mfe_2.output());
}

#[test]
fn pack_unpack_invalid_amount() {
    let bytes = vec![
        42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42,
        42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 0, 82, 253, 252, 7, 33,
        130, 101, 79, 22, 63, 95, 15, 154, 98, 29, 114, 149, 102, 199, 77, 16, 3, 124, 77, 123, 187, 4, 7, 209, 226,
        198, 73, 42, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert!(matches!(
        MigratedFundsEntry::unpack(&mut bytes.as_slice()),
        Err(Error::InvalidMigratedFundsEntryAmount(42))
    ));
}
