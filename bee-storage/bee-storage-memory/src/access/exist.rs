// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Exist access operations.

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
use bee_storage::{access::Exist, backend::StorageBackend};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::storage::Storage;

macro_rules! impl_exist {
    ($key:ty, $value:ty, $field:ident) => {
        impl Exist<$key, $value> for Storage {
            fn exist(&self, k: &$key) -> Result<bool, <Self as StorageBackend>::Error> {
                Ok(self.inner.read()?.$field.exist(k))
            }
        }
    };
}

impl_exist!(BlockId, Block, block_id_to_block);
impl_exist!(BlockId, BlockMetadata, block_id_to_metadata);
impl_exist!((BlockId, BlockId), (), block_id_to_block_id);
impl_exist!(OutputId, CreatedOutput, output_id_to_created_output);
impl_exist!(OutputId, ConsumedOutput, output_id_to_consumed_output);
impl_exist!(Unspent, (), output_id_unspent);
impl_exist!((Ed25519Address, OutputId), (), ed25519_address_to_output_id);
impl_exist!((), LedgerIndex, ledger_index);
impl_exist!(MilestoneIndex, MilestoneMetadata, milestone_index_to_milestone_metadata);
impl_exist!(MilestoneId, MilestonePayload, milestone_id_to_milestone_payload);
impl_exist!((), SnapshotInfo, snapshot_info);
impl_exist!(SolidEntryPoint, MilestoneIndex, solid_entry_point_to_milestone_index);
impl_exist!(MilestoneIndex, OutputDiff, milestone_index_to_output_diff);
impl_exist!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    milestone_index_to_unreferenced_block
);
impl_exist!((MilestoneIndex, Receipt), (), milestone_index_to_receipt);
impl_exist!((bool, TreasuryOutput), (), spent_to_treasury_output);
