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
use bee_storage::access::Delete;
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use packable::PackableExt;

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

impl Delete<BlockId, Block> for Storage {
    fn delete(&self, block_id: &BlockId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.delete_cf(self.cf_handle(CF_BLOCK_ID_TO_BLOCK)?, block_id)?;

        Ok(())
    }
}

impl Delete<BlockId, BlockMetadata> for Storage {
    fn delete(&self, block_id: &BlockId) -> Result<(), <Self as StorageBackend>::Error> {
        let guard = self.locks.block_id_to_metadata.read();

        self.inner
            .delete_cf(self.cf_handle(CF_BLOCK_ID_TO_METADATA)?, block_id)?;

        drop(guard);

        Ok(())
    }
}

impl Delete<(BlockId, BlockId), ()> for Storage {
    fn delete(&self, (parent, child): &(BlockId, BlockId)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner.delete_cf(self.cf_handle(CF_BLOCK_ID_TO_BLOCK_ID)?, key)?;

        Ok(())
    }
}

impl Delete<OutputId, CreatedOutput> for Storage {
    fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?, output_id.pack_to_vec())?;

        Ok(())
    }
}

impl Delete<OutputId, ConsumedOutput> for Storage {
    fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.delete_cf(
            self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?,
            output_id.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Delete<Unspent, ()> for Storage {
    fn delete(&self, unspent: &Unspent) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, unspent.pack_to_vec())?;

        Ok(())
    }
}

impl Delete<(Ed25519Address, OutputId), ()> for Storage {
    fn delete(&self, (address, output_id): &(Ed25519Address, OutputId)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_to_vec());

        self.inner
            .delete_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, key)?;

        Ok(())
    }
}

impl Delete<(), LedgerIndex> for Storage {
    fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.delete_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8])?;

        Ok(())
    }
}

impl Delete<MilestoneIndex, MilestoneMetadata> for Storage {
    fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.delete_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE_METADATA)?,
            index.pack_to_vec(),
        )?;

        Ok(())
    }
}

impl Delete<MilestoneId, MilestonePayload> for Storage {
    fn delete(&self, id: &MilestoneId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?, id.pack_to_vec())?;

        Ok(())
    }
}

impl Delete<(), SnapshotInfo> for Storage {
    fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.delete_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8])?;

        Ok(())
    }
}

impl Delete<SolidEntryPoint, MilestoneIndex> for Storage {
    fn delete(&self, sep: &SolidEntryPoint) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?, sep.as_ref())?;

        Ok(())
    }
}

impl Delete<MilestoneIndex, OutputDiff> for Storage {
    fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?, index.pack_to_vec())?;

        Ok(())
    }
}

impl Delete<(MilestoneIndex, UnreferencedBlock), ()> for Storage {
    fn delete(
        &self,
        (index, unreferenced_block): &(MilestoneIndex, UnreferencedBlock),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(unreferenced_block.as_ref());

        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK)?, key)?;

        Ok(())
    }
}

impl Delete<(MilestoneIndex, Receipt), ()> for Storage {
    fn delete(&self, (index, receipt): &(MilestoneIndex, Receipt)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(&receipt.pack_to_vec());

        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, key)?;

        Ok(())
    }
}

impl Delete<(bool, TreasuryOutput), ()> for Storage {
    fn delete(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_to_vec();
        key.extend_from_slice(&output.pack_to_vec());

        self.inner
            .delete_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, key)?;

        Ok(())
    }
}
