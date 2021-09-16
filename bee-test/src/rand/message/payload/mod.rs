// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod application_message;
mod collective_beacon;
mod data;
mod dkg;
mod fpc;
mod indexation;
mod regular_beacon;
mod salt_declaration;
mod transaction;

pub use application_message::rand_application_message_payload;
pub use collective_beacon::rand_collective_beacon_payload;
pub use data::rand_data_payload;
pub use dkg::rand_dkg_payload;
pub use fpc::{rand_conflict, rand_fpc_payload, rand_opinion, rand_timestamp};
pub use indexation::rand_indexation_payload;
pub use regular_beacon::rand_beacon_payload;
pub use salt_declaration::rand_salt_declaration_payload;
pub use transaction::{rand_transaction_id, rand_transaction_payload};

use crate::rand::number::rand_number;

use bee_message::payload::{
    data::DataPayload,
    drng::{ApplicationMessagePayload, BeaconPayload, CollectiveBeaconPayload, DkgPayload},
    fpc::FpcPayload,
    indexation::IndexationPayload,
    salt_declaration::SaltDeclarationPayload,
    transaction::TransactionPayload,
    MessagePayload, Payload,
};

/// Generates a random [`Payload`].
pub fn rand_payload() -> Payload {
    match rand_number::<u32>() % 9 {
        DataPayload::KIND => rand_data_payload().into(),
        TransactionPayload::KIND => rand_transaction_payload().into(),
        FpcPayload::KIND => rand_fpc_payload().into(),
        ApplicationMessagePayload::KIND => rand_application_message_payload().into(),
        DkgPayload::KIND => rand_dkg_payload().into(),
        BeaconPayload::KIND => rand_beacon_payload().into(),
        CollectiveBeaconPayload::KIND => rand_collective_beacon_payload().into(),
        SaltDeclarationPayload::KIND => rand_salt_declaration_payload().into(),
        IndexationPayload::KIND => rand_indexation_payload().into(),
        _ => unreachable!(),
    }
}
