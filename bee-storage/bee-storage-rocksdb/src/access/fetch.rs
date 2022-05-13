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
};
use bee_storage::{access::Fetch, system::System};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use packable::PackableExt;

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

impl Fetch<u8, System> for Storage {
    fn fetch(&self, key: &u8) -> Result<Option<System>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_SYSTEM)?, [*key])?
            // Unpacking from storage is fine.
            .map(|v| System::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<BlockId, Block> for Storage {
    fn fetch(&self, message_id: &BlockId) -> Result<Option<Block>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id)?
            // Unpacking from storage is fine.
            .map(|v| Block::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<BlockId, BlockMetadata> for Storage {
    fn fetch(&self, message_id: &BlockId) -> Result<Option<BlockMetadata>, <Self as StorageBackend>::Error> {
        let guard = self.locks.message_id_to_metadata.read();

        let metadata = self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?, message_id)?
            // Unpacking from storage is fine.
            .map(|v| BlockMetadata::unpack_unverified(&mut &*v).unwrap());

        drop(guard);

        Ok(metadata)
    }
}

impl Fetch<BlockId, Vec<BlockId>> for Storage {
    fn fetch(&self, parent: &BlockId) -> Result<Option<Vec<BlockId>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .prefix_iterator_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, parent)
                .map(|(key, _)| {
                    let (_, child) = key.split_at(BlockId::LENGTH);
                    // Unpacking from storage is fine.
                    let child: [u8; BlockId::LENGTH] = child.try_into().unwrap();
                    BlockId::from(child)
                })
                .take(self.config.fetch_edge_limit)
                .collect(),
        ))
    }
}

impl Fetch<OutputId, CreatedOutput> for Storage {
    fn fetch(&self, output_id: &OutputId) -> Result<Option<CreatedOutput>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?, output_id.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| CreatedOutput::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<OutputId, ConsumedOutput> for Storage {
    fn fetch(&self, output_id: &OutputId) -> Result<Option<ConsumedOutput>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(
                self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?,
                output_id.pack_to_vec(),
            )?
            // Unpacking from storage is fine.
            .map(|v| ConsumedOutput::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<Ed25519Address, Vec<OutputId>> for Storage {
    fn fetch(&self, address: &Ed25519Address) -> Result<Option<Vec<OutputId>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .prefix_iterator_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, address)
                .map(|(key, _)| {
                    let (_, output_id) = key.split_at(Ed25519Address::LENGTH);
                    // Unpacking from storage is fine.
                    TryFrom::<[u8; OutputId::LENGTH]>::try_from(output_id.try_into().unwrap()).unwrap()
                })
                .take(self.config.fetch_output_id_limit)
                .collect(),
        ))
    }
}

impl Fetch<(), LedgerIndex> for Storage {
    fn fetch(&self, (): &()) -> Result<Option<LedgerIndex>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8])?
            // Unpacking from storage is fine.
            .map(|v| LedgerIndex::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<MilestoneIndex, MilestoneMetadata> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<MilestoneMetadata>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(
                self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE_METADATA)?,
                index.pack_to_vec(),
            )?
            // Unpacking from storage is fine.
            .map(|v| MilestoneMetadata::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<MilestoneId, MilestonePayload> for Storage {
    fn fetch(&self, id: &MilestoneId) -> Result<Option<MilestonePayload>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?, id.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| MilestonePayload::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<(), SnapshotInfo> for Storage {
    fn fetch(&self, (): &()) -> Result<Option<SnapshotInfo>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8])?
            // Unpacking from storage is fine.
            .map(|v| SnapshotInfo::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<SolidEntryPoint, MilestoneIndex> for Storage {
    fn fetch(&self, sep: &SolidEntryPoint) -> Result<Option<MilestoneIndex>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?, sep.as_ref())?
            // Unpacking from storage is fine.
            .map(|v| MilestoneIndex::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<MilestoneIndex, OutputDiff> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<OutputDiff>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_pinned_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?, index.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| OutputDiff::unpack_unverified(&mut &*v).unwrap()))
    }
}

impl Fetch<MilestoneIndex, Vec<UnreferencedBlock>> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Vec<UnreferencedBlock>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .prefix_iterator_cf(
                    self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?,
                    index.pack_to_vec(),
                )
                .map(|(key, _)| {
                    let (_, unreferenced_block) = key.split_at(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    let unreferenced_block: [u8; BlockId::LENGTH] = unreferenced_block.try_into().unwrap();
                    UnreferencedBlock::from(BlockId::from(unreferenced_block))
                })
                .collect(),
        ))
    }
}

impl Fetch<MilestoneIndex, Vec<Receipt>> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Vec<Receipt>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .prefix_iterator_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, index.pack_to_vec())
                .map(|(mut key, _)| {
                    let (_, receipt) = key.split_at_mut(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    #[allow(clippy::useless_asref)]
                    Receipt::unpack_unverified(&mut receipt.as_ref()).unwrap()
                })
                .collect(),
        ))
    }
}

impl Fetch<bool, Vec<TreasuryOutput>> for Storage {
    fn fetch(&self, spent: &bool) -> Result<Option<Vec<TreasuryOutput>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .prefix_iterator_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, spent.pack_to_vec())
                .map(|(mut key, _)| {
                    let (_, output) = key.split_at_mut(std::mem::size_of::<bool>());
                    // Unpacking from storage is fine.
                    #[allow(clippy::useless_asref)]
                    TreasuryOutput::unpack_unverified(&mut output.as_ref()).unwrap()
                })
                .collect(),
        ))
    }
}
