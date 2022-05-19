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
use packable::PackableExt;

use crate::{storage::Storage, trees::*};

impl Insert<u8, System> for Storage {
    fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.insert(&[*key], value.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<BlockId, Block> for Storage {
    fn insert(&self, block_id: &BlockId, block: &Block) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_BLOCK_ID_TO_BLOCK)?
            .insert(block_id, block.pack_to_vec())?;

        Ok(())
    }
}

impl InsertStrict<BlockId, BlockMetadata> for Storage {
    fn insert_strict(
        &self,
        block_id: &BlockId,
        metadata: &BlockMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_BLOCK_ID_TO_METADATA)?
            .update_and_fetch(block_id, |old_metadata| {
                old_metadata
                    .map(|b| b.to_vec())
                    .or_else(|| Some(metadata.pack_to_vec()))
            })?;

        Ok(())
    }
}

impl Insert<(BlockId, BlockId), ()> for Storage {
    fn insert(&self, (parent, child): &(BlockId, BlockId), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner.open_tree(TREE_BLOCK_ID_TO_BLOCK_ID)?.insert(key, &[])?;

        Ok(())
    }
}

impl Insert<OutputId, CreatedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &CreatedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .insert(output_id.pack_to_vec(), output.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<OutputId, ConsumedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &ConsumedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .insert(output_id.pack_to_vec(), output.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<Unspent, ()> for Storage {
    fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_UNSPENT)?
            .insert(unspent.pack_to_vec(), &[])?;

        Ok(())
    }
}

impl Insert<(Ed25519Address, OutputId), ()> for Storage {
    fn insert(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_to_vec());

        self.inner
            .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
            .insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(), LedgerIndex> for Storage {
    fn insert(&self, (): &(), index: &LedgerIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_LEDGER_INDEX)?
            .insert([0x00u8], index.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, MilestoneMetadata> for Storage {
    fn insert(
        &self,
        index: &MilestoneIndex,
        milestone: &MilestoneMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA)?
            .insert(index.pack_to_vec(), milestone.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<MilestoneId, MilestonePayload> for Storage {
    fn insert(&self, id: &MilestoneId, payload: &MilestonePayload) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?
            .insert(id.pack_to_vec(), payload.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<(), SnapshotInfo> for Storage {
    fn insert(&self, (): &(), info: &SnapshotInfo) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_SNAPSHOT_INFO)?
            .insert([0x00u8], info.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<SolidEntryPoint, MilestoneIndex> for Storage {
    fn insert(&self, sep: &SolidEntryPoint, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .insert(sep.as_ref(), index.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, OutputDiff> for Storage {
    fn insert(&self, index: &MilestoneIndex, diff: &OutputDiff) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .insert(index.pack_to_vec(), diff.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<(MilestoneIndex, UnreferencedBlock), ()> for Storage {
    fn insert(
        &self,
        (index, unreferenced_block): &(MilestoneIndex, UnreferencedBlock),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(unreferenced_block.as_ref());

        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK)?
            .insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(MilestoneIndex, Receipt), ()> for Storage {
    fn insert(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(&receipt.pack_to_vec());

        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
            .insert(key, &[])?;

        Ok(())
    }
}

impl Insert<(bool, TreasuryOutput), ()> for Storage {
    fn insert(&self, (spent, output): &(bool, TreasuryOutput), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_to_vec();
        key.extend_from_slice(&output.pack_to_vec());

        self.inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?.insert(key, &[])?;

        Ok(())
    }
}
