// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use core::str::FromStr;

const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn new_valid() {
    let tth = [42; 49];
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let amount = 42424242;

    let mfe = MigratedFundsEntry::new(tth, address, amount).unwrap();

    assert_eq!(mfe.tail_transaction_hash(), &tth);
    assert_eq!(*mfe.address(), address);
    assert_eq!(mfe.amount(), amount);
}

#[test]
fn new_invalid_amount() {
    let tth = [42; 49];
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let amount = 42;

    assert!(matches!(
        MigratedFundsEntry::new(tth, address, amount),
        Err(Error::InvalidMigratedFundsEntryAmount(42))
    ));
}

#[test]
fn pack_unpack_valid() {
    let tth = [42; 49];
    let address = Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap());
    let amount = 42424242;
    let mfe_1 = MigratedFundsEntry::new(tth, address, amount).unwrap();
    let mfe_2 = MigratedFundsEntry::unpack(&mut mfe_1.pack_new().as_slice()).unwrap();

    assert_eq!(mfe_1.tail_transaction_hash(), mfe_2.tail_transaction_hash());
    assert_eq!(*mfe_1.address(), *mfe_2.address());
    assert_eq!(mfe_1.amount(), mfe_1.amount());
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
