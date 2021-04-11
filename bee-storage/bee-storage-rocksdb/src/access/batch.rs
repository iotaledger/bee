// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_common::packable::Packable;
use bee_ledger::{
    snapshot::info::SnapshotInfo,
    types::{Balance, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent},
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::{ConsumedOutput, CreatedOutput, OutputId},
    payload::indexation::HashedIndex,
    Message, MessageId,
};
use bee_storage::access::{Batch, BatchBuilder};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

use rocksdb::{WriteBatch, WriteOptions};

#[derive(Default)]
pub struct StorageBatch {
    inner: WriteBatch,
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

#[async_trait::async_trait]
impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    async fn batch_commit(&self, batch: Self::Batch, durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        let mut write_options = WriteOptions::default();
        write_options.set_sync(false);
        write_options.disable_wal(!durability);
        self.inner.write_opt(batch.inner, &write_options)?;

        Ok(())
    }
}

impl Batch<MessageId, Message> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
        message: &Message,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        batch.value_buf.clear();
        // Packing to bytes can't fail.
        message.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, message_id, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        batch.inner.delete_cf(&cf, message_id);

        Ok(())
    }
}

impl Batch<MessageId, MessageMetadata> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
        metadata: &MessageMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        batch.value_buf.clear();
        // Packing to bytes can't fail.
        metadata.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, message_id, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        batch.inner.delete_cf(&cf, message_id);

        Ok(())
    }
}

impl Batch<(MessageId, MessageId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (parent, child): &(MessageId, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch.inner.put_cf(&cf, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (parent, child): &(MessageId, MessageId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch.inner.delete_cf(&cf, &batch.key_buf);

        Ok(())
    }
}

impl Batch<(HashedIndex, MessageId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (index, message_id): &(HashedIndex, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(index.as_ref());
        batch.key_buf.extend_from_slice(message_id.as_ref());

        batch.inner.put_cf(&cf, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, message_id): &(HashedIndex, MessageId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(index.as_ref());
        batch.key_buf.extend_from_slice(message_id.as_ref());

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CREATED_OUTPUT))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        output.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, &batch.key_buf, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CREATED_OUTPUT))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        output.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, &batch.key_buf, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        output_id: &OutputId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        output_id.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf, &batch.key_buf);

        Ok(())
    }
}

impl Batch<Unspent, ()> for Storage {
    fn batch_insert(&self, batch: &mut Self::Batch, unspent: &Unspent, (): &()) -> Result<(), Self::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_UNSPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_UNSPENT))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        unspent.pack(&mut batch.key_buf).unwrap();

        batch.inner.put_cf(&cf, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, unspent: &Unspent) -> Result<(), Self::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_UNSPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_UNSPENT))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        unspent.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(address.as_ref());
        batch.key_buf.extend_from_slice(&output_id.pack_new());

        batch.inner.put_cf(&cf, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(address.as_ref());
        batch.key_buf.extend_from_slice(&output_id.pack_new());

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        batch.value_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, [0x00u8], &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        batch.inner.delete_cf(&cf, [0x00u8]);

        Ok(())
    }
}

impl Batch<MilestoneIndex, Milestone> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
        milestone: &Milestone,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_MILESTONE))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        milestone.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, &batch.key_buf, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_MILESTONE))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        batch.value_buf.clear();
        // Packing to bytes can't fail.
        info.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, [0x00u8], &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        batch.inner.delete_cf(&cf, [0x00u8]);

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
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        sep.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, &batch.key_buf, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, sep: &SolidEntryPoint) -> Result<(), Self::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        sep.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        diff.pack(&mut batch.value_buf).unwrap();

        batch.inner.put_cf(&cf, &batch.key_buf, &batch.value_buf);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF))?;

        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();

        batch.inner.delete_cf(&cf, &batch.key_buf);

        Ok(())
    }
}

impl Batch<Address, Balance> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        address: &Address,
        balance: &Balance,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ADDRESS_TO_BALANCE)
            .ok_or(Error::UnknownCf(CF_ADDRESS_TO_BALANCE))?;

        batch.inner.put_cf(&cf, address.pack_new(), balance.pack_new());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, address: &Address) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ADDRESS_TO_BALANCE)
            .ok_or(Error::UnknownCf(CF_ADDRESS_TO_BALANCE))?;

        batch.inner.delete_cf(&cf, address.pack_new());

        Ok(())
    }
}

impl Batch<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(unconfirmed_message.as_ref());

        batch.inner.put_cf(&cf, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(unconfirmed_message.as_ref());

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_RECEIPT))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(&receipt.pack_new());

        batch.inner.put_cf(&cf, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, receipt): &(MilestoneIndex, Receipt),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_RECEIPT))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(&receipt.pack_new());

        batch.inner.delete_cf(&cf, &batch.key_buf);

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
        let cf = self
            .inner
            .cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)
            .ok_or(Error::UnknownCf(CF_SPENT_TO_TREASURY_OUTPUT))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&spent.pack_new());
        batch.key_buf.extend_from_slice(&output.pack_new());

        batch.inner.put_cf(&cf, &batch.key_buf, []);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (spent, output): &(bool, TreasuryOutput),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)
            .ok_or(Error::UnknownCf(CF_SPENT_TO_TREASURY_OUTPUT))?;

        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&spent.pack_new());
        batch.key_buf.extend_from_slice(&output.pack_new());

        batch.inner.delete_cf(&cf, &batch.key_buf);

        Ok(())
    }
}
