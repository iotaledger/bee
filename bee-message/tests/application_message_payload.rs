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
fn unpack_valid() {
    let bytes = vec![0, 0, 0, 1];

    assert!(ApplicationMessagePayload::unpack_from_slice(bytes).is_ok());
}

#[test]
fn accessors_eq() {
    let application_msg = ApplicationMessagePayload::new(1);

    assert_eq!(application_msg.instance_id(), 1);
}

#[test]
fn packed_len() {
    let application_msg = ApplicationMessagePayload::new(1);

    assert_eq!(application_msg.packed_len(), 4);
}

#[test]
fn packable_round_trip() {
    let application_msg_a = ApplicationMessagePayload::new(1);

    let application_msg_b =
        ApplicationMessagePayload::unpack_from_slice(application_msg_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(application_msg_a, application_msg_b);
}
