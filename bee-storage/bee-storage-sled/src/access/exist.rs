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
use packable::PackableExt;

use crate::{storage::Storage, trees::*};

impl Exist<BlockId, Block> for Storage {
    fn exist(&self, block_id: &BlockId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.open_tree(TREE_BLOCK_ID_TO_BLOCK)?.contains_key(block_id)?)
    }
}

impl Exist<BlockId, BlockMetadata> for Storage {
    fn exist(&self, block_id: &BlockId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_BLOCK_ID_TO_METADATA)?
            .contains_key(block_id)?)
    }
}

impl Exist<(BlockId, BlockId), ()> for Storage {
    fn exist(&self, (parent, child): &(BlockId, BlockId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self.inner.open_tree(TREE_BLOCK_ID_TO_BLOCK_ID)?.contains_key(key)?)
    }
}

impl Exist<OutputId, CreatedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .contains_key(output_id.pack_to_vec())?)
    }
}

impl Exist<OutputId, ConsumedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .contains_key(output_id.pack_to_vec())?)
    }
}

impl Exist<Unspent, ()> for Storage {
    fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_UNSPENT)?
            .contains_key(unspent.pack_to_vec())?)
    }
}

impl Exist<(Ed25519Address, OutputId), ()> for Storage {
    fn exist(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_to_vec());

        Ok(self
            .inner
            .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
            .contains_key(key)?)
    }
}

impl Exist<(), LedgerIndex> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.open_tree(TREE_LEDGER_INDEX)?.contains_key([0x00u8])?)
    }
}

impl Exist<MilestoneIndex, MilestoneMetadata> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA)?
            .contains_key(index.pack_to_vec())?)
    }
}

impl Exist<MilestoneId, MilestonePayload> for Storage {
    fn exist(&self, index: &MilestoneId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?
            .contains_key(index.pack_to_vec())?)
    }
}

impl Exist<(), SnapshotInfo> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.open_tree(TREE_SNAPSHOT_INFO)?.contains_key([0x00u8])?)
    }
}

impl Exist<SolidEntryPoint, MilestoneIndex> for Storage {
    fn exist(&self, sep: &SolidEntryPoint) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .contains_key(sep.pack_to_vec())?)
    }
}

impl Exist<MilestoneIndex, OutputDiff> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .contains_key(index.pack_to_vec())?)
    }
}

impl Exist<(MilestoneIndex, UnreferencedBlock), ()> for Storage {
    fn exist(
        &self,
        (index, unreferenced_block): &(MilestoneIndex, UnreferencedBlock),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(unreferenced_block.as_ref());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK)?
            .contains_key(key)?)
    }
}

impl Exist<(MilestoneIndex, Receipt), ()> for Storage {
    fn exist(&self, (index, receipt): &(MilestoneIndex, Receipt)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(&receipt.pack_to_vec());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
            .contains_key(key)?)
    }
}

impl Exist<(bool, TreasuryOutput), ()> for Storage {
    fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = spent.pack_to_vec();
        key.extend_from_slice(&output.pack_to_vec());

        Ok(self.inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?.contains_key(key)?)
    }
}
