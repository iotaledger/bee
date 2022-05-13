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
use bee_storage::access::{Batch, BatchBuilder};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use packable::{Packable, PackableExt};
use rocksdb::{WriteBatch, WriteOptions};

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

#[derive(Default)]
pub struct StorageBatch {
    should_lock: bool,
    inner: WriteBatch,
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    fn batch_commit(&self, batch: Self::Batch, durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);

        let guard = batch.should_lock.then(|| self.locks.message_id_to_metadata.read());

        self.inner.write_opt(batch.inner, &write_options)?;

        drop(guard);

        Ok(())
    }
}

impl Batch<BlockId, Block> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &BlockId,
        message: &Block,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        message.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .put_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &BlockId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id);

        Ok(())
    }
}

impl Batch<BlockId, BlockMetadata> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &BlockId,
        metadata: &BlockMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.should_lock = true;

        batch.value_buf.clear();
        // Packing to bytes can't fail.
        metadata.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .put_cf(self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?, message_id, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &BlockId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.should_lock = true;

        batch
            .inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?, message_id);

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
            .put_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, &batch.key_buf, []);

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
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, &batch.key_buf);

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

        batch.inner.put_cf(
            self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?,
            &batch.key_buf,
            &batch.value_buf,
        );

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
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?, &batch.key_buf);

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

        batch.inner.put_cf(
            self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?,
            &batch.key_buf,
            &batch.value_buf,
        );

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
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?, &batch.key_buf);

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
            .put_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, unspent: &Unspent) -> Result<(), Self::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        unspent.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, &batch.key_buf);

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
            .put_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, &batch.key_buf, []);

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
            .delete_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, &batch.key_buf);

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
            .put_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8], &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        batch.inner.delete_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8]);

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

        batch.inner.put_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE_METADATA)?,
            &batch.key_buf,
            &batch.value_buf,
        );

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

        batch.inner.delete_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE_METADATA)?,
            &batch.key_buf,
        );

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

        batch.inner.put_cf(
            self.cf_handle(CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?,
            &batch.key_buf,
            &batch.value_buf,
        );

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, id: &MilestoneId) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        id.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .delete_cf(self.cf_handle(CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD)?, &batch.key_buf);

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
            .put_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8], &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        batch.inner.delete_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8]);

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

        batch.inner.put_cf(
            self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?,
            &batch.key_buf,
            &batch.value_buf,
        );

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, sep: &SolidEntryPoint) -> Result<(), Self::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        sep.pack(&mut batch.key_buf).unwrap();

        batch
            .inner
            .delete_cf(self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?, &batch.key_buf);

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

        batch.inner.put_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?,
            &batch.key_buf,
            &batch.value_buf,
        );

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
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?, &batch.key_buf);

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

        batch.inner.put_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?,
            &batch.key_buf,
            [],
        );

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

        batch.inner.delete_cf(
            self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?,
            &batch.key_buf,
        );

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
            .put_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, &batch.key_buf, []);

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
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, &batch.key_buf);

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
            .put_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, &batch.key_buf, []);

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
            .delete_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, &batch.key_buf);

        Ok(())
    }
}
