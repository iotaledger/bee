// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
    system::System,
};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use packable::PackableExt;

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

impl Insert<u8, System> for Storage {
    fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_SYSTEM)?, [*key], value.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<BlockId, Block> for Storage {
    fn insert(&self, block_id: &BlockId, block: &Block) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?,
            block_id,
            block.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl InsertStrict<BlockId, BlockMetadata> for Storage {
    fn insert_strict(
        &self,
        block_id: &BlockId,
        metadata: &BlockMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let guard = self.locks.block_id_to_metadata.read();

        self.inner.merge_cf(
            self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?,
            block_id,
            metadata.pack_to_vec(),
        )?;

        drop(guard);

        Ok(())
    }
}

impl Insert<(BlockId, BlockId), ()> for Storage {
    fn insert(&self, (parent, child): &(BlockId, BlockId), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner
            .put_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, key, [])?;

        Ok(())
    }
}

impl Insert<OutputId, CreatedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &CreatedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?,
            output_id.pack_to_vec(),
            output.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<OutputId, ConsumedOutput> for Storage {
    fn insert(&self, output_id: &OutputId, output: &ConsumedOutput) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?,
            output_id.pack_to_vec(),
            output.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<Unspent, ()> for Storage {
    fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, unspent.pack_to_vec(), [])?;

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
            .put_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, key, [])?;

        Ok(())
    }
}

impl Insert<(), LedgerIndex> for Storage {
    fn insert(&self, (): &(), index: &LedgerIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8], index.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, MilestoneMetadata> for Storage {
    fn insert(
        &self,
        index: &MilestoneIndex,
        milestone: &MilestoneMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE_METADATA)?,
            index.pack_to_vec(),
            milestone.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<MilestoneId, MilestonePayload> for Storage {
    fn insert(&self, id: &MilestoneId, payload: &MilestonePayload) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?,
            id.pack_to_vec(),
            payload.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<(), SnapshotInfo> for Storage {
    fn insert(&self, (): &(), info: &SnapshotInfo) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .put_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8], info.pack_to_vec())?;

        Ok(())
    }
}

impl Insert<SolidEntryPoint, MilestoneIndex> for Storage {
    fn insert(&self, sep: &SolidEntryPoint, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?,
            sep.as_ref(),
            index.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Insert<MilestoneIndex, OutputDiff> for Storage {
    fn insert(&self, index: &MilestoneIndex, diff: &OutputDiff) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.put_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?,
            index.pack_to_vec(),
            diff.pack_to_vec(),
        )?;

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
            .put_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?, key, [])?;

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
            .put_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, key, [])?;

        Ok(())
    }
}

impl Insert<(bool, TreasuryOutput), ()> for Storage {
    fn insert(&self, (spent, output): &(bool, TreasuryOutput), (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_to_vec();
        key.extend_from_slice(&output.pack_to_vec());

        self.inner
            .put_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, key, [])?;

        Ok(())
    }
}
