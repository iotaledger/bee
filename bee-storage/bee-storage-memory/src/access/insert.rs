// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Insert access operations.

use bee_block::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{
    snapshot::info::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
    Unspent,
};
use bee_storage::{
    access::{Insert, InsertStrict},
    backend::StorageBackend,
    system::System,
};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::storage::Storage;

macro_rules! impl_insert {
    ($key:ty, $value:ty, $field:ident) => {
        impl Insert<$key, $value> for Storage {
            fn insert(&self, k: &$key, v: &$value) -> Result<(), <Self as StorageBackend>::Error> {
                self.inner.write()?.$field.insert(k, v);

                Ok(())
            }
        }
    };
}

impl_insert!(u8, System, system);
impl_insert!(BlockId, Block, message_id_to_message);
impl_insert!((BlockId, BlockId), (), message_id_to_message_id);
impl_insert!(OutputId, CreatedOutput, output_id_to_created_output);
impl_insert!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_insert!(Unspent, (), output_id_unspent);
impl_insert!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_insert!((), LedgerIndex, ledger_index);
impl_insert!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_insert!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_insert!((), SnapshotInfo, snapshot_info);
impl_insert!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_insert!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_insert!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    milestone_index_to_unreferenced_block
);
impl_insert!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_insert!((bool, TreasuryOutput), (), spent_to_treasury_output);

impl InsertStrict<BlockId, BlockMetadata> for Storage {
    fn insert_strict(&self, k: &BlockId, v: &BlockMetadata) -> Result<(), <Self as StorageBackend>::Error> {
        let mut guard = self.inner.write()?;

        if !guard.message_id_to_metadata.exist(k) {
            guard.message_id_to_metadata.insert(k, v);
        }

        drop(guard);

        Ok(())
    }
}
