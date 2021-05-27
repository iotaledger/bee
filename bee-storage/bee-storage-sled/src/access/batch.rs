// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{storage::Storage, trees::*};

use bee_common::packable::Packable;
use bee_ledger::types::{
    snapshot::info::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt,
    TreasuryOutput, Unspent,
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::{
    access::{Batch, BatchBuilder},
    backend::StorageBackend,
};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

use sled;

use std::collections::HashMap;

#[derive(Default)]
pub struct StorageBatch {
    inner: HashMap<&'static str, sled::Batch>,
    key_buf: Vec<u8>,
    value_buf: Vec<u8>,
}

#[async_trait::async_trait]
impl BatchBuilder for Storage {
    type Batch = StorageBatch;

    async fn batch_commit(&self, batch: Self::Batch, _durability: bool) -> Result<(), <Self as StorageBackend>::Error> {
        for (tree, batch) in batch.inner {
            self.inner.open_tree(tree)?.apply_batch(batch)?;
        }

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
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        message.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_MESSAGE)
            .or_default()
            .insert(message_id.as_ref(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_MESSAGE)
            .or_default()
            .remove(message_id.as_ref());

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
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        metadata.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_METADATA)
            .or_default()
            .insert(message_id.as_ref(), batch.value_buf.as_slice());

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        message_id: &MessageId,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_METADATA)
            .or_default()
            .remove(message_id.as_ref());

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
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_MESSAGE_ID)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (parent, child): &(MessageId, MessageId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(parent.as_ref());
        batch.key_buf.extend_from_slice(child.as_ref());

        batch
            .inner
            .entry(TREE_MESSAGE_ID_TO_MESSAGE_ID)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}

impl Batch<(PaddedIndex, MessageId), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (index, message_id): &(PaddedIndex, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(index.as_ref());
        batch.key_buf.extend_from_slice(message_id.as_ref());

        batch
            .inner
            .entry(TREE_INDEX_TO_MESSAGE_ID)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, message_id): &(PaddedIndex, MessageId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(index.as_ref());
        batch.key_buf.extend_from_slice(message_id.as_ref());

        batch
            .inner
            .entry(TREE_INDEX_TO_MESSAGE_ID)
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
        batch.key_buf.extend_from_slice(&output_id.pack_new());

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
        batch.key_buf.extend_from_slice(&output_id.pack_new());

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

impl Batch<MilestoneIndex, Milestone> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        index: &MilestoneIndex,
        milestone: &Milestone,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        // Packing to bytes can't fail.
        index.pack(&mut batch.key_buf).unwrap();
        batch.value_buf.clear();
        // Packing to bytes can't fail.
        milestone.pack(&mut batch.value_buf).unwrap();

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_MILESTONE)
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
            .entry(TREE_MILESTONE_INDEX_TO_MILESTONE)
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

impl Batch<Address, Balance> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        address: &Address,
        balance: &Balance,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .entry(TREE_ADDRESS_TO_BALANCE)
            .or_default()
            .insert(address.pack_new(), balance.pack_new());

        Ok(())
    }

    fn batch_delete(&self, batch: &mut Self::Batch, address: &Address) -> Result<(), <Self as StorageBackend>::Error> {
        batch
            .inner
            .entry(TREE_ADDRESS_TO_BALANCE)
            .or_default()
            .remove(address.pack_new());

        Ok(())
    }
}

impl Batch<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    fn batch_insert(
        &self,
        batch: &mut Self::Batch,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(unreferenced_message.as_ref());

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)
            .or_default()
            .insert(batch.key_buf.as_slice(), &[]);

        Ok(())
    }

    fn batch_delete(
        &self,
        batch: &mut Self::Batch,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        batch.key_buf.clear();
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(unreferenced_message.as_ref());

        batch
            .inner
            .entry(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)
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
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(&receipt.pack_new());

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
        batch.key_buf.extend_from_slice(&index.pack_new());
        batch.key_buf.extend_from_slice(&receipt.pack_new());

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
        batch.key_buf.extend_from_slice(&spent.pack_new());
        batch.key_buf.extend_from_slice(&output.pack_new());

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
        batch.key_buf.extend_from_slice(&spent.pack_new());
        batch.key_buf.extend_from_slice(&output.pack_new());

        batch
            .inner
            .entry(TREE_SPENT_TO_TREASURY_OUTPUT)
            .or_default()
            .remove(batch.key_buf.as_slice());

        Ok(())
    }
}
