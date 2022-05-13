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
use bee_storage::access::Exist;
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use packable::PackableExt;

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

impl Exist<BlockId, Block> for Storage {
    fn exist(&self, block_id: &BlockId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, block_id)?
            .is_some())
    }
}

impl Exist<BlockId, BlockMetadata> for Storage {
    fn exist(&self, block_id: &BlockId) -> Result<bool, <Self as StorageBackend>::Error> {
        let guard = self.locks.block_id_to_metadata.read();

        let exists = self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?, block_id)?
            .is_some();

        drop(guard);

        Ok(exists)
    }
}

impl Exist<(BlockId, BlockId), ()> for Storage {
    fn exist(&self, (parent, child): &(BlockId, BlockId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, key)?
            .is_some())
    }
}

impl Exist<OutputId, CreatedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?, output_id.pack_to_vec())?
            .is_some())
    }
}

impl Exist<OutputId, ConsumedOutput> for Storage {
    fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(
                self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?,
                output_id.pack_to_vec(),
            )?
            .is_some())
    }
}

impl Exist<Unspent, ()> for Storage {
    fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, unspent.pack_to_vec())?
            .is_some())
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
            .get_pinned_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, key)?
            .is_some())
    }
}

impl Exist<(), LedgerIndex> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8])?
            .is_some())
    }
}

impl Exist<MilestoneIndex, MilestoneMetadata> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(
                self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE_METADATA)?,
                index.pack_to_vec(),
            )?
            .is_some())
    }
}

impl Exist<MilestoneId, MilestonePayload> for Storage {
    fn exist(&self, id: &MilestoneId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?, id.pack_to_vec())?
            .is_some())
    }
}

impl Exist<(), SnapshotInfo> for Storage {
    fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8])?
            .is_some())
    }
}

impl Exist<SolidEntryPoint, MilestoneIndex> for Storage {
    fn exist(&self, sep: &SolidEntryPoint) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(
                self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?,
                sep.pack_to_vec(),
            )?
            .is_some())
    }
}

impl Exist<MilestoneIndex, OutputDiff> for Storage {
    fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?, index.pack_to_vec())?
            .is_some())
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
            .get_pinned_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?, key)?
            .is_some())
    }
}

impl Exist<(MilestoneIndex, Receipt), ()> for Storage {
    fn exist(&self, (index, receipt): &(MilestoneIndex, Receipt)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_to_vec();
        key.extend_from_slice(&receipt.pack_to_vec());

        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, key)?
            .is_some())
    }
}

impl Exist<(bool, TreasuryOutput), ()> for Storage {
    fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = spent.pack_to_vec();
        key.extend_from_slice(&output.pack_to_vec());

        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, key)?
            .is_some())
    }
}
