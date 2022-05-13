// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Iter access operations.

use bee_block::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_storage::{access::AsIterator, backend::StorageBackend, system::System};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::{
    storage::Storage,
    table::{SingletonTableIter, TableIter, VecTableIter},
};

macro_rules! impl_iter {
    (($key:ty, $value:ty), (), $field:ident) => {
        impl<'a> AsIterator<'a, ($key, $value), ()> for Storage {
            type AsIter = VecTableIter<$key, $value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.iter())
            }
        }
    };

    ((), $value:ty, $field:ident) => {
        impl<'a> AsIterator<'a, (), $value> for Storage {
            type AsIter = SingletonTableIter<$value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.iter())
            }
        }
    };
    ($key:ty, $value:ty, $field:ident) => {
        impl<'a> AsIterator<'a, $key, $value> for Storage {
            type AsIter = TableIter<$key, $value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.iter())
            }
        }
    };
}

impl_iter!(u8, System, system);
impl_iter!(BlockId, Block, message_id_to_message);
impl_iter!(BlockId, BlockMetadata, message_id_to_metadata);
impl_iter!((BlockId, BlockId), (), message_id_to_message_id);
impl_iter!(OutputId, CreatedOutput, output_id_to_created_output);
impl_iter!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_iter!(Unspent, (), output_id_unspent);
impl_iter!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_iter!((), LedgerIndex, ledger_index);
impl_iter!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_iter!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_iter!((), SnapshotInfo, snapshot_info);
impl_iter!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_iter!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_iter!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    milestone_index_to_unreferenced_block
);
impl_iter!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_iter!((bool, TreasuryOutput), (), spent_to_treasury_output);
