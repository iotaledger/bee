// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Stream access operations.
pub use crate::table::VecTableIter;

use crate::{
    storage::Storage,
    table::{SingletonTableIter, TableIter},
};

use bee_ledger::types::{
    snapshot::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
    Unspent,
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::{access::AsIterator, backend::StorageBackend, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

macro_rules! impl_stream {
    (($key:ty, $value:ty), (), $field:ident) => {
        impl<'a> AsIterator<'a, ($key, $value), ()> for Storage {
            type AsIter = VecTableIter<$key, $value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                self.$field.iter()
            }
        }
    };

    ((), $value:ty, $field:ident) => {
        impl<'a> AsIterator<'a, (), $value> for Storage {
            type AsIter = SingletonTableIter<$value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                self.$field.iter()
            }
        }
    };
    ($key:ty, $value:ty, $field:ident) => {
        impl<'a> AsIterator<'a, $key, $value> for Storage {
            type AsIter = TableIter<$key, $value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                self.$field.iter()
            }
        }
    };
}

impl_stream!(u8, System, system);
impl_stream!(MessageId, Message, message_id_to_message);
impl_stream!(MessageId, MessageMetadata, message_id_to_metadata);
impl_stream!((MessageId, MessageId), (), message_id_to_message_id);
impl_stream!((PaddedIndex, MessageId), (), index_to_message_id);
impl_stream!(OutputId, CreatedOutput, output_id_to_created_output);
impl_stream!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_stream!(Unspent, (), output_id_unspent);
impl_stream!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_stream!((), LedgerIndex, ledger_index);
impl_stream!(MilestoneIndex, Milestone, milestone_index_to_milestone);
impl_stream!((), SnapshotInfo, snapshot_info);
impl_stream!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_stream!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_stream!(Address, Balance, address_to_balance);
impl_stream!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    milestone_index_to_unreferenced_message
);
impl_stream!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_stream!((bool, TreasuryOutput), (), spent_to_treasury_output);
