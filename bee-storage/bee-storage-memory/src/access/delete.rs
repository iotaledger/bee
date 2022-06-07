// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Delete access operations.

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
use bee_storage::{access::Delete, backend::StorageBackend};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::storage::Storage;

macro_rules! impl_delete {
    ($key:ty, $value:ty, $field:ident) => {
        impl Delete<$key, $value> for Storage {
            fn delete(&self, k: &$key) -> Result<(), <Self as StorageBackend>::Error> {
                self.inner.write()?.$field.delete(k);

                Ok(())
            }
        }
    };
}

impl_delete!(BlockId, Block, block_id_to_block);
impl_delete!(BlockId, BlockMetadata, block_id_to_metadata);
impl_delete!((BlockId, BlockId), (), block_id_to_block_id);
impl_delete!(OutputId, CreatedOutput, output_id_to_created_output);
impl_delete!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_delete!(Unspent, (), output_id_unspent);
impl_delete!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_delete!((), LedgerIndex, ledger_index);
impl_delete!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_delete!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_delete!((), SnapshotInfo, snapshot_info);
impl_delete!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_delete!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_delete!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    milestone_index_to_unreferenced_block
);
impl_delete!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_delete!((bool, TreasuryOutput), (), spent_to_treasury_output);
