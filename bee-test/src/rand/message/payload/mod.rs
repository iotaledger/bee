// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Module providing random (FPC) conflict utilities.
pub mod conflict;
/// Module providing random salt generation utilities.
pub mod salt;
/// Module providing randoom (FPC) timestamp utilities.
pub mod timestamp;
/// Module providing random transaction generation utilities.
pub mod transaction;

use crate::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    message::payload::{conflict::rand_conflict, salt::rand_salt, timestamp::rand_timestamp},
    number::{rand_number, rand_number_range},
    vec::rand_vec,
};

use bee_message::payload::{
    data::DataPayload,
    drng::{ApplicationMessagePayload, BeaconPayload, CollectiveBeaconPayload, DkgPayload, EncryptedDeal},
    fpc::FpcPayload,
    indexation::{IndexationPayload, INDEXATION_INDEX_LENGTH_RANGE},
    salt_declaration::SaltDeclarationPayload,
};

/// Generates a random [`ApplicationMessagePayload`].
pub fn rand_application_message_payload() -> ApplicationMessagePayload {
    ApplicationMessagePayload::new(rand_number())
}

/// Generates a random [`BeaconPayload`].
pub fn rand_beacon_payload() -> BeaconPayload {
    BeaconPayload::builder()
        .with_instance_id(rand_number())
        .with_round(rand_number())
        .with_partial_public_key(rand_bytes_array())
        .with_partial_signature(rand_bytes_array())
        .finish()
        .unwrap()
}

/// Generates a random [`CollectiveBeaconPayload`].
pub fn rand_collective_beacon_payload() -> CollectiveBeaconPayload {
    CollectiveBeaconPayload::builder()
        .with_instance_id(rand_number())
        .with_round(rand_number())
        .with_prev_signature(rand_bytes_array())
        .with_signature(rand_bytes_array())
        .with_distributed_public_key(rand_bytes_array())
        .finish()
        .unwrap()
}

/// Generates a random [`DataPayload`].
pub fn rand_data_payload() -> DataPayload {
    DataPayload::new(rand_bytes(rand_number_range(0..=255))).unwrap()
}

/// Generates a random [`DkgPayload`].
pub fn rand_dkg_payload() -> DkgPayload {
    DkgPayload::builder()
        .with_instance_id(rand_number())
        .with_from_index(rand_number())
        .with_to_index(rand_number())
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(rand_bytes(rand_number_range(0..=127)))
                .with_nonce(rand_bytes(rand_number_range(0..=127)))
                .with_encrypted_share(rand_bytes(rand_number_range(0..=127)))
                .with_threshold(rand_number())
                .with_commitments(rand_bytes(rand_number_range(0..=127)))
                .finish()
                .unwrap(),
        )
        .finish()
        .unwrap()
}

/// Generates a random [`IndexationPayload`].
pub fn rand_indexation_payload() -> IndexationPayload {
    let index_range = *INDEXATION_INDEX_LENGTH_RANGE.start() as usize..=*INDEXATION_INDEX_LENGTH_RANGE.end() as usize;

    IndexationPayload::new(
        rand_bytes(rand_number_range(index_range)),
        rand_bytes(rand_number_range(0..=255)),
    )
    .unwrap()
}

/// Generates a random [`FpcPayload`].
pub fn rand_fpc_payload() -> FpcPayload {
    FpcPayload::builder()
        .with_conflicts(rand_vec(rand_conflict, rand_number_range(0..=10)))
        .with_timestamps(rand_vec(rand_timestamp, rand_number_range(0..=10)))
        .finish()
        .unwrap()
}

/// Generates a random [`SaltDeclarationPayload`].
pub fn rand_salt_declaration_payload() -> SaltDeclarationPayload {
    SaltDeclarationPayload::builder()
        .with_node_id(rand_number())
        .with_salt(rand_salt())
        .with_timestamp(rand_number())
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap()
}
