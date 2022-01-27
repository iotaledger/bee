// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, Ed25519Address},
    input::{Input, UtxoInput},
    output::{unlock_condition::AddressUnlockCondition, ExtendedOutput, Output},
    payload::transaction::{RegularTransactionEssence, TransactionEssence, TransactionId},
    Error,
};

use packable::{error::UnpackError, PackableExt};

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
    let output = Output::Extended(
        ExtendedOutput::build(amount)
            .add_unlock_condition(AddressUnlockCondition::new(address).into())
            .finish()
            .unwrap(),
    );
    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder()
            .with_inputs(vec![input1, input2])
            .add_output(output)
            .finish()
            .unwrap(),
    );

    assert_eq!(essence.kind(), RegularTransactionEssence::KIND);
}

#[test]
fn essence_unpack_invalid_kind() {
    assert!(matches!(
        TransactionEssence::unpack_verified(&mut vec![1u8; 32].as_slice()),
        Err(UnpackError::Packable(Error::InvalidEssenceKind(1)))
    ));
}
