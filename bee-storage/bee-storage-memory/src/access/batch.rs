// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Batch access operations.

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
    access::{Batch, BatchBuilder},
    backend::StorageBackend,
};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::{storage::Storage, table::TableBatch};

/// A writing batch that can be applied atomically.
#[derive(Default)]
pub struct StorageBatch {
    block_id_to_block: TableBatch<BlockId, Block>,
    block_id_to_metadata: TableBatch<BlockId, BlockMetadata>,
    block_id_to_block_id: TableBatch<(BlockId, BlockId), ()>,
    output_id_to_created_output: TableBatch<OutputId, CreatedOutput>,
    output_id_to_consumed_output: TableBatch<OutputId, ConsumedOutput>,
    output_id_unspent: TableBatch<Unspent, ()>,
    ed25519_address_to_output_id: TableBatch<(Ed25519Address, OutputId), ()>,
    ledger_index: TableBatch<(), LedgerIndex>,
    milestone_index_to_milestone_metadata: TableBatch<MilestoneIndex, MilestoneMetadata>,
    milestone_id_to_milestone_payload: TableBatch<MilestoneId, MilestonePayload>,
    snapshot_info: TableBatch<(), SnapshotInfo>,
    solid_entry_point_to_milestone_index: TableBatch<SolidEntryPoint, MilestoneIndex>,
    milestone_index_to_output_diff: TableBatch<MilestoneIndex, OutputDiff>,
    milestone_index_to_unreferenced_block: TableBatch<(MilestoneIndex, UnreferencedBlock), ()>,
    milestone_index_to_receipt: TableBatch<(MilestoneIndex, Receipt), ()>,
    spent_to_treasury_output: TableBatch<(bool, TreasuryOutput), ()>,
}

impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    fn batch_begin() -> Self::Batch {
        Self::Batch::default()
    }

    fn batch_commit(&self, batch: Self::Batch, _durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        let mut inner = self.inner.write()?;

        macro_rules! apply_batch {
            ($field:ident) => {
                inner.$field.batch_commit(batch.$field);
            };
        }

        apply_batch!(block_id_to_block);
        apply_batch!(block_id_to_metadata);
        apply_batch!(block_id_to_block_id);
        apply_batch!(output_id_to_created_output);
        apply_batch!(output_id_to_consumed_output);
        apply_batch!(output_id_unspent);
        apply_batch!(ed25519_address_to_output_id);
        apply_batch!(ledger_index);
        apply_batch!(milestone_index_to_milestone_metadata);
        apply_batch!(milestone_id_to_milestone_payload);
        apply_batch!(snapshot_info);
        apply_batch!(solid_entry_point_to_milestone_index);
        apply_batch!(milestone_index_to_output_diff);
        apply_batch!(milestone_index_to_unreferenced_block);
        apply_batch!(milestone_index_to_receipt);
        apply_batch!(spent_to_treasury_output);

        Ok(())
    }
}

macro_rules! impl_batch {
    ($key:ty, $value:ty, $field:ident) => {
        impl Batch<$key, $value> for Storage {
            fn batch_insert(
                &self,
                batch: &mut Self::Batch,
                key: &$key,
                value: &$value,
            ) -> Result<(), <Self as StorageBackend>::Error> {
                batch.$field.insert(key, value);

                Ok(())
            }

            fn batch_delete(&self, batch: &mut Self::Batch, key: &$key) -> Result<(), <Self as StorageBackend>::Error> {
                batch.$field.delete(key);

                Ok(())
            }
        }
    };
}

impl_batch!(BlockId, Block, block_id_to_block);
impl_batch!(BlockId, BlockMetadata, block_id_to_metadata);
impl_batch!((BlockId, BlockId), (), block_id_to_block_id);
impl_batch!(OutputId, CreatedOutput, output_id_to_created_output);
impl_batch!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_batch!(Unspent, (), output_id_unspent);
impl_batch!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_batch!((), LedgerIndex, ledger_index);
impl_batch!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_batch!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_batch!((), SnapshotInfo, snapshot_info);
impl_batch!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_batch!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_batch!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    milestone_index_to_unreferenced_block
);
impl_batch!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_batch!((bool, TreasuryOutput), (), spent_to_treasury_output);
