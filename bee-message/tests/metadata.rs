// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageMetadata;
use bee_packable::packable::Packable;
use bee_test::rand::{
    bytes::rand_bytes_array,
    message::{metadata::rand_message_metadata, payload::rand_opinion},
    number::rand_number,
};

#[test]
fn flags() {
    let mut metadata = MessageMetadata::new(0);

    assert!(!metadata.flags().is_solid());
    assert!(!metadata.flags().is_scheduled());
    assert!(!metadata.flags().is_booked());
    assert!(!metadata.flags().is_eligible());
    assert!(!metadata.flags().is_invalid());

    metadata.flags_mut().set_solid(true);
    metadata.flags_mut().set_scheduled(true);
    metadata.flags_mut().set_booked(true);
    metadata.flags_mut().set_eligible(true);
    metadata.flags_mut().set_invalid(true);

    assert!(metadata.flags().is_solid());
    assert!(metadata.flags().is_scheduled());
    assert!(metadata.flags().is_booked());
    assert!(metadata.flags().is_eligible());
    assert!(metadata.flags().is_invalid());
}

#[test]
fn accessors_eq() {
    let received_timestamp = rand_number();
    let solidification_timestamp = rand_number();
    let branch_id = rand_bytes_array();
    let opinion = rand_opinion();
    let mut metadata = MessageMetadata::new(received_timestamp);

    metadata.set_solidification_timestamp(solidification_timestamp);
    metadata.set_branch_id(branch_id);
    metadata.set_opinion(opinion);

    assert_eq!(metadata.received_timestamp(), received_timestamp);
    assert_eq!(metadata.solidification_timestamp(), solidification_timestamp);
    assert_eq!(metadata.branch_id(), &branch_id);
    assert_eq!(metadata.opinion(), &opinion);
}

#[test]
fn packed_len() {
    assert_eq!(rand_message_metadata().packed_len(), 1 + 8 + 8 + 32 + 1);
}

#[test]
fn packable_round_trip() {
    let metadata_a = rand_message_metadata();
    let metadata_b = MessageMetadata::unpack_from_slice(metadata_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(metadata_a, metadata_b);
}
