// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;

use std::convert::TryInto;

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn essence_kind() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());
    let essence = Essence::Regular(
        RegularEssence::builder()
            .with_inputs(vec![input1, input2])
            .with_outputs(vec![output])
            .finish()
            .unwrap(),
    );

    assert_eq!(essence.kind(), RegularEssence::KIND);
}

#[test]
fn essence_unpack_invalid_kind() {
    assert!(matches!(
        Essence::unpack(&mut vec![1u8; 32].as_slice()),
        Err(Error::InvalidEssenceKind(1))
    ));
}
