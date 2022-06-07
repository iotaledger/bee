// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Batch access operations.

use std::{collections::BTreeMap, convert::Infallible};

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
    access::{Batch, BatchBuilder},
    backend::StorageBackend,
};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use packable::{Packable, PackableExt};
use sled::{transaction::TransactionError, Transactional};

use crate::{storage::Storage, trees::*};

/// A writing batch that can be applied atomically.
#[derive(Default)]
pub struct StorageBatch {
    inner: BTreeMap<&'static str, sled::Batch>,
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    fn batch_commit(&self, batch: Self::Batch, _durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        let trees = batch
            .inner
            .keys()
            .map(|tree| self.inner.open_tree(tree))
            .collect::<Result<Vec<_>, _>>()?;

        let transaction_result = Transactional::<Infallible>::transaction::<_, ()>(trees.as_slice(), |trees| {
            for (tree, batch) in trees.iter().zip(batch.inner.values()) {
                tree.apply_batch(batch)?;
            }

            Ok(())
        });

        if let Err(err) = transaction_result {
            match err {
                TransactionError::Storage(err) => {
                    return Err(Self::Error::Sled(err));
                }
                TransactionError::Abort(err) => match err {},
            }
        }

        Ok(())
    }
}

impl Batch<BlockId, Block> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        block_id: &BlockId,
        block: &Block,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        block.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_BLOCK_ID_TO_BLOCK)
            .or_default()
            .insert(block_id.as_ref(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, block_id: &BlockId) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .entry(TREE_BLOCK_ID_TO_BLOCK)
            .or_default()
            .remove(block_id.as_ref());

        Ok(())
    }
}

impl Batch<BlockId, BlockMetadata> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        block_id: &BlockId,
        metadata: &BlockMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        metadata.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_BLOCK_ID_TO_METADATA)
            .or_default()
            .insert(block_id.as_ref(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, block_id: &BlockId) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .entry(TREE_BLOCK_ID_TO_METADATA)
            .or_default()
            .remove(block_id.as_ref());

        Ok(())
    }
}

impl Batch<(BlockId, BlockId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (parent, child): &(BlockId, BlockId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch
            .inner
            .entry(TREE_BLOCK_ID_TO_BLOCK_ID)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (parent, child): &(BlockId, BlockId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch
            .inner
            .entry(TREE_BLOCK_ID_TO_BLOCK_ID)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<OutputId, CreatedOutput> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
        output: &CreatedOutput,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        output.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)
            .or_default()
            .insert(batch.key_buf.as_slice(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<OutputId, ConsumedOutput> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
        output: &ConsumedOutput,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        output.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .or_default()
            .insert(batch.key_buf.as_slice(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<Unspent, ()> for Storage {
    fn batch_insert(&self, batch: &mut Self::Batch, unspent: &Unspent, (): &()) -> Result<(), Self::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        unspent.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_OUTPUT_ID_UNSPENT)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, unspent: &Unspent) -> Result<(), Self::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        unspent.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_OUTPUT_ID_UNSPENT)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<(Ed25519Address, OutputId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (address, output_id): &(Ed25519Address, OutputId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(address.as_ref());
        batch.key_buf.extend_from_slice(&output_id.pack_to_vec());

        batch
            .inner
            .entry(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(address.as_ref());
        batch.key_buf.extend_from_slice(&output_id.pack_to_vec());

        batch
            .inner
            .entry(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<(), LedgerIndex> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (): &(),
        index: &LedgerIndex,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_LEDGER_INDEX)
            .or_default()
            .insert(&[0x00u8], batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        batch.inner.entry(TREE_LEDGER_INDEX).or_default().remove(&[0x00u8]);

        Ok(())
    }
}

impl Batch<MilestoneIndex, MilestoneMetadata> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
        milestone: &MilestoneMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        milestone.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA)
            .or_default()
            .insert(batch.key_buf.as_slice(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<MilestoneId, MilestonePayload> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        id: &MilestoneId,
        payload: &MilestonePayload,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        id.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        payload.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD)
            .or_default()
            .insert(batch.key_buf.as_slice(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, id: &MilestoneId) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        id.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<(), SnapshotInfo> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (): &(),
        info: &SnapshotInfo,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        info.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_SNAPSHOT_INFO)
            .or_default()
            .insert(&[0x00u8], batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        batch.inner.entry(TREE_SNAPSHOT_INFO).or_default().remove(&[0x00u8]);

        Ok(())
    }
}

impl Batch<SolidEntryPoint, MilestoneIndex> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        sep: &SolidEntryPoint,
        index: &MilestoneIndex,
    ) -> Result<(), Self::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        sep.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .or_default()
            .insert(batch.key_buf.as_slice(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, sep: &SolidEntryPoint) -> Result<(), Self::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        sep.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<MilestoneIndex, OutputDiff> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
        diff: &OutputDiff,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        diff.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .or_default()
            .insert(batch.key_buf.as_slice(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<(MilestoneIndex, UnreferencedBlock), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (index, unreferenced_block): &(MilestoneIndex, UnreferencedBlock),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_to_vec());
        batch.key_buf.extend_from_slice(unreferenced_block.as_ref());

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, unreferenced_block): &(MilestoneIndex, UnreferencedBlock),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_to_vec());
        batch.key_buf.extend_from_slice(unreferenced_block.as_ref());

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<(MilestoneIndex, Receipt), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (index, receipt): &(MilestoneIndex, Receipt),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_to_vec());
        batch.key_buf.extend_from_slice(&receipt.pack_to_vec());

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_RECEIPT)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, receipt): &(MilestoneIndex, Receipt),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_to_vec());
        batch.key_buf.extend_from_slice(&receipt.pack_to_vec());

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_RECEIPT)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<(bool, TreasuryOutput), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (spent, output): &(bool, TreasuryOutput),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&spent.pack_to_vec());
        batch.key_buf.extend_from_slice(&output.pack_to_vec());

        batch
            .inner
            .entry(TREE_SPENT_TO_TREASURY_OUTPUT)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (spent, output): &(bool, TreasuryOutput),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&spent.pack_to_vec());
        batch.key_buf.extend_from_slice(&output.pack_to_vec());

        batch
            .inner
            .entry(TREE_SPENT_TO_TREASURY_OUTPUT)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}
