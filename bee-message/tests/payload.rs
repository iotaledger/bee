// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, Ed25519Address},
    error::{MessageUnpackError, ValidationError},
    input::{Input, UtxoInput},
    output::{Output, OutputId, SignatureLockedSingleOutput},
    payload::{
        data::DataPayload,
        drng::{ApplicationMessagePayload, BeaconPayload, CollectiveBeaconPayload, DkgPayload, EncryptedDeal},
        fpc::{Conflict, FpcPayload, Timestamp},
        indexation::IndexationPayload,
        salt_declaration::{Salt, SaltDeclarationPayload},
        transaction::{TransactionEssence, TransactionId, TransactionPayload},
        MessagePayload, Payload, PayloadUnpackError,
    },
    signature::{Ed25519Signature, Signature},
    unlock::{SignatureUnlock, UnlockBlock, UnlockBlocks},
    MessageId,
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    number::rand_number,
};

#[test]
fn application_message_payload_packable_round_trip() {
    let payload_1 = Payload::from(ApplicationMessagePayload::new(1));

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), ApplicationMessagePayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn collective_beacon_payload_packable_round_trip() {
    let payload_1 = Payload::from(
        CollectiveBeaconPayload::builder()
            .with_instance_id(0)
            .with_round(1)
            .with_prev_signature(rand_bytes_array())
            .with_signature(rand_bytes_array())
            .with_distributed_public_key(rand_bytes_array())
            .finish()
            .unwrap(),
    );

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), CollectiveBeaconPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn data_payload_packable_round_trip() {
    let payload_1 = Payload::from(DataPayload::new(vec![0; 255]).unwrap());

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), DataPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn dkg_payload_packable_round_trip() {
    let payload_1 = Payload::from(
        DkgPayload::builder()
            .with_instance_id(1)
            .with_from_index(20)
            .with_to_index(32)
            .with_deal(
                EncryptedDeal::builder()
                    .with_dh_key(rand_bytes(128))
                    .with_nonce(rand_bytes(12))
                    .with_encrypted_share(rand_bytes(128))
                    .with_threshold(10)
                    .with_commitments(rand_bytes(12))
                    .finish()
                    .unwrap(),
            )
            .finish()
            .unwrap(),
    );

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), DkgPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn fpc_payload_packable_round_trip() {
    let payload_1 = Payload::from(
        FpcPayload::builder()
            .with_conflicts(vec![
                Conflict::new(TransactionId::from(rand_bytes_array()), 0, 0),
                Conflict::new(TransactionId::from(rand_bytes_array()), 0, 1),
                Conflict::new(TransactionId::from(rand_bytes_array()), 1, 2),
            ])
            .with_timestamps(vec![
                Timestamp::new(MessageId::from(rand_bytes_array()), 0, 0),
                Timestamp::new(MessageId::from(rand_bytes_array()), 0, 1),
                Timestamp::new(MessageId::from(rand_bytes_array()), 1, 2),
            ])
            .finish()
            .unwrap(),
    );

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), FpcPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn indexation_payload_packable_round_trip() {
    let payload_1 = Payload::from(IndexationPayload::new(rand_bytes(32), rand_bytes(64)).unwrap());

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), IndexationPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn regular_beacon_payload_packable_round_trip() {
    let payload_1 = Payload::from(
        BeaconPayload::builder()
            .with_instance_id(0)
            .with_round(1)
            .with_partial_public_key(rand_bytes_array())
            .with_partial_signature(rand_bytes_array())
            .finish()
            .unwrap(),
    );

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), BeaconPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn salt_declaration_payload_packable_round_trip() {
    let payload_1 = Payload::from(
        SaltDeclarationPayload::builder()
            .with_node_id(32)
            .with_salt(Salt::new(rand_bytes(64), rand_number()).unwrap())
            .with_timestamp(rand_number())
            .with_signature(rand_bytes_array())
            .finish()
            .unwrap(),
    );

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), SaltDeclarationPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn transaction_payload_packable_round_trip() {
    let payload_1 = Payload::from(
        TransactionPayload::builder()
            .with_essence(
                TransactionEssence::builder()
                    .with_timestamp(rand_number())
                    .with_access_pledge_id(rand_bytes_array())
                    .with_consensus_pledge_id(rand_bytes_array())
                    .with_inputs(vec![Input::Utxo(UtxoInput::new(
                        OutputId::new(TransactionId::new(rand_bytes_array()), 0).unwrap(),
                    ))])
                    .with_outputs(vec![Output::SignatureLockedSingle(
                        SignatureLockedSingleOutput::new(
                            Address::from(Ed25519Address::new(rand_bytes_array())),
                            1_000_000,
                        )
                        .unwrap(),
                    )])
                    .finish()
                    .unwrap(),
            )
            .with_unlock_blocks(
                UnlockBlocks::new(vec![UnlockBlock::Signature(SignatureUnlock::from(Signature::Ed25519(
                    Ed25519Signature::new(rand_bytes_array(), rand_bytes_array()),
                )))])
                .unwrap(),
            )
            .finish()
            .unwrap(),
    );

    let bytes = payload_1.pack_to_vec().unwrap();
    let payload_2 = Payload::unpack_from_slice(bytes.clone()).unwrap();

    assert_eq!(payload_1, payload_2);
    assert_eq!(payload_1.kind(), TransactionPayload::KIND);
    assert_eq!(payload_1.packed_len(), bytes.len());
}

#[test]
fn unpack_invalid_version() {
    let mut bytes = vec![0x08, 0x00, 0x00, 0x00, 0x01];
    bytes.extend([0x20, 0x00, 0x00, 0x00]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend([0x40, 0x00, 0x00, 0x00]);
    bytes.extend(rand_bytes_array::<64>());

    assert!(matches!(
        Payload::unpack_from_slice(bytes).err().unwrap(),
        UnpackError::Packable(MessageUnpackError::Validation(ValidationError::InvalidPayloadVersion {
            version: 1,
            payload_kind: 8,
        }))
    ));
}

#[test]
fn unpack_invalid_kind() {
    let mut bytes = vec![0x12, 0x00, 0x00, 0x00, 0x00];
    bytes.extend([0x20, 0x00, 0x00, 0x00]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend([0x40, 0x00, 0x00, 0x00]);
    bytes.extend(rand_bytes_array::<64>());

    assert!(matches!(
        Payload::unpack_from_slice(bytes).err().unwrap(),
        UnpackError::Packable(MessageUnpackError::Payload(PayloadUnpackError::InvalidKind(18))),
    ));
}
