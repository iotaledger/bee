// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    payload::{indexation::IndexationPayload, Payload},
    Message, MessageBuilder, MessageUnpackError, ValidationError,
};
use bee_packable::{Packable, UnpackError};
use bee_test::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    message::{parents::rand_parents, payload::rand_indexation_payload},
    number::rand_number,
};

#[test]
fn new_valid() {
    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish();

    assert!(message.is_ok());
}

#[test]
fn unpack_valid() {
    let bytes = vec![
        1, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 238, 142, 220, 195, 72, 99, 77, 135,
        73, 71, 196, 160, 101, 213, 130, 203, 214, 96, 245, 30, 3, 44, 37, 103, 128, 55, 240, 155, 139, 220, 142, 178,
        216, 230, 192, 191, 209, 104, 112, 20, 2, 0, 0, 0, 109, 0, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 132, 27, 114, 220,
        115, 116, 126, 193, 10, 134, 212, 173, 149, 101, 177, 183, 239, 215, 196, 68, 91, 60, 110, 222, 214, 229, 233,
        139, 78, 192, 242, 72, 64, 0, 0, 0, 153, 128, 64, 149, 20, 34, 176, 142, 218, 58, 195, 204, 46, 40, 206, 2, 5,
        166, 147, 196, 253, 226, 199, 30, 119, 83, 20, 169, 249, 80, 123, 20, 163, 123, 208, 238, 69, 191, 198, 110,
        105, 107, 184, 244, 12, 51, 64, 199, 121, 8, 14, 248, 38, 118, 144, 2, 133, 4, 126, 169, 122, 117, 124, 134, 0,
        0, 0, 0, 0, 0, 0, 0, 145, 167, 69, 239, 139, 44, 177, 36, 175, 85, 127, 123, 121, 5, 53, 252, 47, 72, 99, 133,
        46, 48, 76, 67, 166, 136, 216, 171, 49, 120, 150, 197, 94, 234, 36, 251, 59, 102, 43, 196, 54, 55, 138, 254,
        248, 226, 27, 75, 64, 65, 70, 179, 143, 249, 27, 85, 91, 169, 46, 237, 98, 213, 205, 27,
    ];

    let message = Message::unpack_from_slice(bytes);

    assert!(message.is_ok());
}

#[test]
fn unpack_invalid_version() {
    let bytes = vec![
        0, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 238, 142, 220, 195, 72, 99, 77, 135,
        73, 71, 196, 160, 101, 213, 130, 203, 214, 96, 245, 30, 3, 44, 37, 103, 128, 55, 240, 155, 139, 220, 142, 178,
        216, 230, 192, 191, 209, 104, 112, 20, 2, 0, 0, 0, 109, 0, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 132, 27, 114, 220,
        115, 116, 126, 193, 10, 134, 212, 173, 149, 101, 177, 183, 239, 215, 196, 68, 91, 60, 110, 222, 214, 229, 233,
        139, 78, 192, 242, 72, 64, 0, 0, 0, 153, 128, 64, 149, 20, 34, 176, 142, 218, 58, 195, 204, 46, 40, 206, 2, 5,
        166, 147, 196, 253, 226, 199, 30, 119, 83, 20, 169, 249, 80, 123, 20, 163, 123, 208, 238, 69, 191, 198, 110,
        105, 107, 184, 244, 12, 51, 64, 199, 121, 8, 14, 248, 38, 118, 144, 2, 133, 4, 126, 169, 122, 117, 124, 134, 0,
        0, 0, 0, 0, 0, 0, 0, 145, 167, 69, 239, 139, 44, 177, 36, 175, 85, 127, 123, 121, 5, 53, 252, 47, 72, 99, 133,
        46, 48, 76, 67, 166, 136, 216, 171, 49, 120, 150, 197, 94, 234, 36, 251, 59, 102, 43, 196, 54, 55, 138, 254,
        248, 226, 27, 75, 64, 65, 70, 179, 143, 249, 27, 85, 91, 169, 46, 237, 98, 213, 205, 27,
    ];

    let message = Message::unpack_from_slice(bytes);

    assert!(matches!(
        message,
        Err(UnpackError::Packable(MessageUnpackError::Validation(
            ValidationError::InvalidMessageVersion(0)
        )))
    ));
}

#[test]
fn unpack_invalid_payload_length() {
    let bytes = vec![
        1, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 238, 142, 220, 195, 72, 99, 77, 135,
        73, 71, 196, 160, 101, 213, 130, 203, 214, 96, 245, 30, 3, 44, 37, 103, 128, 55, 240, 155, 139, 220, 142, 178,
        216, 230, 192, 191, 209, 104, 112, 20, 2, 0, 0, 0, 108, 0, 0, 0, 8, 0, 0, 0, 0, 32, 0, 0, 0, 132, 27, 114, 220,
        115, 116, 126, 193, 10, 134, 212, 173, 149, 101, 177, 183, 239, 215, 196, 68, 91, 60, 110, 222, 214, 229, 233,
        139, 78, 192, 242, 72, 64, 0, 0, 0, 153, 128, 64, 149, 20, 34, 176, 142, 218, 58, 195, 204, 46, 40, 206, 2, 5,
        166, 147, 196, 253, 226, 199, 30, 119, 83, 20, 169, 249, 80, 123, 20, 163, 123, 208, 238, 69, 191, 198, 110,
        105, 107, 184, 244, 12, 51, 64, 199, 121, 8, 14, 248, 38, 118, 144, 2, 133, 4, 126, 169, 122, 117, 124, 134, 0,
        0, 0, 0, 0, 0, 0, 0, 145, 167, 69, 239, 139, 44, 177, 36, 175, 85, 127, 123, 121, 5, 53, 252, 47, 72, 99, 133,
        46, 48, 76, 67, 166, 136, 216, 171, 49, 120, 150, 197, 94, 234, 36, 251, 59, 102, 43, 196, 54, 55, 138, 254,
        248, 226, 27, 75, 64, 65, 70, 179, 143, 249, 27, 85, 91, 169, 46, 237, 98, 213, 205, 27,
    ];

    assert!(matches!(
        Message::unpack_from_slice(bytes),
        Err(UnpackError::Packable(MessageUnpackError::Validation(
            ValidationError::PayloadLengthMismatch {
                expected: 108,
                actual: 109,
            }
        ))),
    ));
}

#[test]
fn accessors_eq() {
    let issuer_public_key = rand_bytes_array::<32>();
    let issue_timestamp = rand_number::<u64>();
    let sequence_number = rand_number::<u32>();
    let payload = Payload::from(IndexationPayload::new(rand_bytes(32), rand_bytes(32)).unwrap());
    let nonce = 0;
    let signature = rand_bytes_array::<64>();

    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(issuer_public_key)
        .with_issue_timestamp(issue_timestamp)
        .with_sequence_number(sequence_number)
        .with_payload(payload.clone())
        .with_nonce(nonce)
        .with_signature(signature)
        .finish()
        .unwrap();

    assert_eq!(*message.issuer_public_key(), issuer_public_key);
    assert_eq!(message.issue_timestamp(), issue_timestamp);
    assert_eq!(message.sequence_number(), sequence_number);
    assert_eq!(*message.payload().unwrap(), payload);
    assert_eq!(message.nonce(), nonce);
    assert_eq!(*message.signature(), signature);
}

#[test]
fn packed_len() {
    let parents = rand_parents();

    let message = MessageBuilder::new()
        .with_parents(parents.clone())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap();

    assert_eq!(
        message.packed_len(),
        1 + 1 + 1 + 32 * parents.len() + 32 + 8 + 4 + 4 + 4 + 1 + 4 + 32 + 4 + 256 + 8 + 64,
    );
}

#[test]
fn packable_round_trip() {
    let message_a = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap();

    let message_b = Message::unpack_from_slice(message_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(message_a, message_b);
}

#[test]
fn serde_round_trip() {
    let message_1 = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(Payload::from(
            IndexationPayload::new(rand_bytes(32), rand_bytes(256)).unwrap(),
        ))
        .with_nonce(0)
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap();
    let json = serde_json::to_string(&message_1).unwrap();
    let message_2 = serde_json::from_str::<Message>(&json).unwrap();

    assert_eq!(message_1, message_2);
}

#[test]
fn builder_missing_parents() {
    let message = MessageBuilder::new()
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(rand_indexation_payload().into())
        .with_nonce(rand_number())
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(message, Err(ValidationError::MissingBuilderField("parents")),));
}

#[test]
fn builder_missing_issuer_public_key() {
    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(rand_indexation_payload().into())
        .with_nonce(rand_number())
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(
        message,
        Err(ValidationError::MissingBuilderField("issuer_public_key")),
    ));
}

#[test]
fn builder_missing_issue_timestamp() {
    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_sequence_number(rand_number())
        .with_payload(rand_indexation_payload().into())
        .with_nonce(rand_number())
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(
        message,
        Err(ValidationError::MissingBuilderField("issue_timestamp")),
    ));
}

#[test]
fn builder_missing_sequence_number() {
    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_payload(rand_indexation_payload().into())
        .with_nonce(rand_number())
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(
        message,
        Err(ValidationError::MissingBuilderField("sequence_number")),
    ));
}

#[test]
fn builder_missing_nonce() {
    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(rand_indexation_payload().into())
        .with_signature(rand_bytes_array())
        .finish();

    assert!(matches!(message, Err(ValidationError::MissingBuilderField("nonce")),));
}

#[test]
fn builder_missing_signature() {
    let message = MessageBuilder::new()
        .with_parents(rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number())
        .with_payload(rand_indexation_payload().into())
        .with_nonce(rand_number())
        .finish();

    assert!(matches!(
        message,
        Err(ValidationError::MissingBuilderField("signature")),
    ));
}
