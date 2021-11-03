// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::{drng::ApplicationMessagePayload, MessagePayload};
use bee_packable::Packable;

#[test]
fn kind() {
    assert_eq!(ApplicationMessagePayload::KIND, 3);
}

#[test]
fn version() {
    assert_eq!(ApplicationMessagePayload::VERSION, 0);
}

#[test]
fn new() {
    let application_msg = ApplicationMessagePayload::new(1);

    assert_eq!(application_msg.instance_id(), 1);
}

#[test]
fn from() {
    let application_msg = ApplicationMessagePayload::from(1);

    assert_eq!(application_msg.instance_id(), 1);
}

#[test]
fn unpack_valid() {
    let bytes = vec![0, 0, 0, 1];

    assert!(ApplicationMessagePayload::unpack_from_slice(bytes).is_ok());
}

#[test]
fn packed_len() {
    let application_msg = ApplicationMessagePayload::new(1);

    assert_eq!(application_msg.packed_len(), 4);
}

#[test]
fn packable_round_trip() {
    let application_msg_1 = ApplicationMessagePayload::new(1);
    let application_msg_2 = ApplicationMessagePayload::unpack_from_slice(application_msg_1.pack_to_vec()).unwrap();

    assert_eq!(application_msg_1, application_msg_2);
}

#[test]
fn serde_round_trip() {
    let application_payload_1 = ApplicationMessagePayload::from(42);
    let json = serde_json::to_string(&application_payload_1).unwrap();
    let application_payload_2 = serde_json::from_str::<ApplicationMessagePayload>(&json).unwrap();

    assert_eq!(application_payload_1, application_payload_2);
}
