// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Truncate access operations.

use bee_block::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_storage::{access::Truncate, backend::StorageBackend};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};

use crate::{storage::Storage, trees::*};

macro_rules! impl_truncate {
    ($key:ty, $value:ty, $cf:expr) => {
        impl Truncate<$key, $value> for Storage {
            fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
                self.inner.drop_tree($cf)?;

                Ok(())
            }
        }
    };
}

impl_truncate!(BlockId, Block, TREE_BLOCK_ID_TO_BLOCK);
impl_truncate!(BlockId, BlockMetadata, TREE_BLOCK_ID_TO_METADATA);
impl_truncate!((BlockId, BlockId), (), TREE_BLOCK_ID_TO_BLOCK_ID);
impl_truncate!(OutputId, CreatedOutput, TREE_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_truncate!(OutputId, ConsumedOutput, TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_truncate!(Unspent, (), TREE_OUTPUT_ID_UNSPENT);
impl_truncate!((Ed25519Address, OutputId), (), TREE_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_truncate!((), LedgerIndex, TREE_LEDGER_INDEX);
impl_truncate!(
    MilestoneIndex,
    MilestoneMetadata,
    TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA
);
impl_truncate!(MilestoneId, MilestonePayload, TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD);
impl_truncate!((), SnapshotInfo, TREE_SNAPSHOT_INFO);
impl_truncate!(
    SolidEntryPoint,
    MilestoneIndex,
    TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX
);
impl_truncate!(MilestoneIndex, OutputDiff, TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_truncate!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    TREE_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK
);
impl_truncate!((MilestoneIndex, Receipt), (), TREE_MILESTONE_INDEX_TO_RECEIPT);
impl_truncate!((bool, TreasuryOutput), (), TREE_SPENT_TO_TREASURY_OUTPUT);
