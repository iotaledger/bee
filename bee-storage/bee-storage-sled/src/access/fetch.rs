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
use packable::PackableExt;

use crate::{storage::Storage, trees::*};

impl Fetch<u8, System> for Storage {
    fn fetch(&self, &key: &u8) -> Result<Option<System>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get([key])?
            // Unpacking from storage is fine.
            .map(|v| System::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<BlockId, Block> for Storage {
    fn fetch(&self, block_id: &BlockId) -> Result<Option<Block>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_BLOCK_ID_TO_BLOCK)?
            .get(block_id)?
            // Unpacking from storage is fine.
            .map(|v| Block::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<BlockId, BlockMetadata> for Storage {
    fn fetch(&self, block_id: &BlockId) -> Result<Option<BlockMetadata>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_BLOCK_ID_TO_METADATA)?
            .get(block_id)?
            // Unpacking from storage is fine.
            .map(|v| BlockMetadata::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<BlockId, Vec<BlockId>> for Storage {
    fn fetch(&self, parent: &BlockId) -> Result<Option<Vec<BlockId>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_BLOCK_ID_TO_BLOCK_ID)?
                .scan_prefix(parent)
                .map(|result| {
                    let (key, _) = result?;
                    let (_, child) = key.split_at(BlockId::LENGTH);
                    // Unpacking from storage is fine.
                    let child: [u8; BlockId::LENGTH] = child.try_into().unwrap();
                    Ok(BlockId::from(child))
                })
                .take(self.config.storage.fetch_edge_limit)
                .collect::<Result<Vec<BlockId>, Self::Error>>()?,
        ))
    }
}

impl Fetch<OutputId, CreatedOutput> for Storage {
    fn fetch(&self, output_id: &OutputId) -> Result<Option<CreatedOutput>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .get(output_id.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| CreatedOutput::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<OutputId, ConsumedOutput> for Storage {
    fn fetch(&self, output_id: &OutputId) -> Result<Option<ConsumedOutput>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .get(output_id.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| ConsumedOutput::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<Ed25519Address, Vec<OutputId>> for Storage {
    fn fetch(&self, address: &Ed25519Address) -> Result<Option<Vec<OutputId>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
                .scan_prefix(address)
                .map(|result| {
                    let (key, _) = result?;
                    let (_, output_id) = key.split_at(Ed25519Address::LENGTH);
                    // Unpacking from storage is fine.
                    Ok((<[u8; OutputId::LENGTH]>::try_from(output_id).unwrap())
                        .try_into()
                        .unwrap())
                })
                .take(self.config.storage.fetch_output_id_limit)
                .collect::<Result<Vec<OutputId>, Self::Error>>()?,
        ))
    }
}

impl Fetch<(), LedgerIndex> for Storage {
    fn fetch(&self, (): &()) -> Result<Option<LedgerIndex>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_LEDGER_INDEX)?
            .get([0x00u8])?
            // Unpacking from storage is fine.
            .map(|v| LedgerIndex::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<MilestoneIndex, MilestoneMetadata> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<MilestoneMetadata>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA)?
            .get(index.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| MilestoneMetadata::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<MilestoneId, MilestonePayload> for Storage {
    fn fetch(&self, id: &MilestoneId) -> Result<Option<MilestonePayload>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?
            .get(id.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| MilestonePayload::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<(), SnapshotInfo> for Storage {
    fn fetch(&self, (): &()) -> Result<Option<SnapshotInfo>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_SNAPSHOT_INFO)?
            .get([0x00u8])?
            // Unpacking from storage is fine.
            .map(|v| SnapshotInfo::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<SolidEntryPoint, MilestoneIndex> for Storage {
    fn fetch(&self, sep: &SolidEntryPoint) -> Result<Option<MilestoneIndex>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .get(sep.as_ref())?
            // Unpacking from storage is fine.
            .map(|v| MilestoneIndex::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<MilestoneIndex, OutputDiff> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<OutputDiff>, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .get(index.pack_to_vec())?
            // Unpacking from storage is fine.
            .map(|v| OutputDiff::unpack_unverified(v.as_ref()).unwrap()))
    }
}

impl Fetch<MilestoneIndex, Vec<UnreferencedBlock>> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Vec<UnreferencedBlock>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK)?
                .scan_prefix(index.pack_to_vec())
                .map(|result| {
                    let (key, _) = result?;
                    let (_, unreferenced_block) = key.split_at(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    let unreferenced_block: [u8; BlockId::LENGTH] = unreferenced_block.try_into().unwrap();
                    Ok(UnreferencedBlock::from(BlockId::from(unreferenced_block)))
                })
                .collect::<Result<Vec<UnreferencedBlock>, Self::Error>>()?,
        ))
    }
}

impl Fetch<MilestoneIndex, Vec<Receipt>> for Storage {
    fn fetch(&self, index: &MilestoneIndex) -> Result<Option<Vec<Receipt>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
                .scan_prefix(index.pack_to_vec())
                .map(|result| {
                    let (mut key, _) = result?;
                    let (_, receipt) = key.split_at_mut(std::mem::size_of::<MilestoneIndex>());
                    // Unpacking from storage is fine.
                    #[allow(clippy::useless_asref)]
                    Ok(Receipt::unpack_unverified(receipt.as_ref()).unwrap())
                })
                .collect::<Result<Vec<Receipt>, Self::Error>>()?,
        ))
    }
}

impl Fetch<bool, Vec<TreasuryOutput>> for Storage {
    fn fetch(&self, spent: &bool) -> Result<Option<Vec<TreasuryOutput>>, <Self as StorageBackend>::Error> {
        Ok(Some(
            self.inner
                .open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?
                .scan_prefix(spent.pack_to_vec())
                .map(|result| {
                    let (mut key, _) = result?;
                    let (_, output) = key.split_at_mut(std::mem::size_of::<bool>());
                    // Unpacking from storage is fine.
                    #[allow(clippy::useless_asref)]
                    Ok(TreasuryOutput::unpack_unverified(output.as_ref()).unwrap())
                })
                .collect::<Result<Vec<TreasuryOutput>, Self::Error>>()?,
        ))
    }
}
