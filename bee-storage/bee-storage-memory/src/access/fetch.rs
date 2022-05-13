// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Fetch access operations.

use bee_block::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{
    snapshot::info::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
};
use bee_storage::{access::Fetch, backend::StorageBackend, system::System};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::storage::Storage;

macro_rules! impl_fetch {
    ($key:ty, $value:ty, $field:ident) => {
        impl Fetch<$key, $value> for Storage {
            fn fetch(&self, k: &$key) -> Result<Option<$value>, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.fetch(k))
            }
        }
    };
}

impl_fetch!(u8, System, system);
impl_fetch!(BlockId, Block, message_id_to_message);
impl_fetch!(BlockId, BlockMetadata, message_id_to_metadata);
impl_fetch!(BlockId, Vec<BlockId>, message_id_to_message_id);
impl_fetch!(OutputId, CreatedOutput, output_id_to_created_output);
impl_fetch!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_fetch!(Ed25519Address, Vec<OutputId>, ed25519_address_to_output_id);
impl_fetch!((), LedgerIndex, ledger_index);
impl_fetch!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_fetch!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_fetch!((), SnapshotInfo, snapshot_info);
impl_fetch!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_fetch!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_fetch!(
    MilestoneIndex,
    Vec<UnreferencedBlock>,
    milestone_index_to_unreferenced_block
);
impl_fetch!(MilestoneIndex, Vec<Receipt>, milestone_index_to_receipt);
impl_fetch!(bool, Vec<TreasuryOutput>, spent_to_treasury_output);
