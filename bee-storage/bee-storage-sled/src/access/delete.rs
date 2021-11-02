// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Delete access operations.

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
use bee_storage::{access::Delete, backend::StorageBackend};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

impl Delete<MessageId, Message> for Storage {
    fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?.remove(message_id)?;

        Ok(())
    }
}

impl Delete<MessageId, MessageMetadata> for Storage {
    fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.open_tree(TREE_MESSAGE_ID_TO_METADATA)?.remove(message_id)?;

        Ok(())
    }
}

impl Delete<(MessageId, MessageId), ()> for Storage {
    fn delete(&self, (parent, child): &(MessageId, MessageId)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE_ID)?.remove(key)?;

        Ok(())
    }
}

impl Delete<(PaddedIndex, MessageId), ()> for Storage {
    fn delete(&self, (index, message_id): &(PaddedIndex, MessageId)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.open_tree(TREE_INDEX_TO_MESSAGE_ID)?.remove(key)?;

        Ok(())
    }
}

impl Delete<OutputId, CreatedOutput> for Storage {
    fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .remove(output_id.pack_new())?;

        Ok(())
    }
}

impl Delete<OutputId, ConsumedOutput> for Storage {
    fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .remove(output_id.pack_new())?;

        Ok(())
    }
}

impl Delete<Unspent, ()> for Storage {
    fn delete(&self, unspent: &Unspent) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_OUTPUT_ID_UNSPENT)?
            .remove(unspent.pack_new())?;

        Ok(())
    }
}

impl Delete<(Ed25519Address, OutputId), ()> for Storage {
    fn delete(&self, (address, output_id): &(Ed25519Address, OutputId)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        self.inner.open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?.remove(key)?;

        Ok(())
    }
}

impl Delete<(), LedgerIndex> for Storage {
    fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.open_tree(TREE_LEDGER_INDEX)?.remove([0x00u8])?;

        Ok(())
    }
}

impl Delete<MilestoneIndex, Milestone> for Storage {
    fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE)?
            .remove(index.pack_new())?;

        Ok(())
    }
}

impl Delete<(), SnapshotInfo> for Storage {
    fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.open_tree(TREE_SNAPSHOT_INFO)?.remove([0x00u8])?;

        Ok(())
    }
}

impl Delete<SolidEntryPoint, MilestoneIndex> for Storage {
    fn delete(&self, sep: &SolidEntryPoint) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .remove(sep.as_ref())?;

        Ok(())
    }
}

impl Delete<MilestoneIndex, OutputDiff> for Storage {
    fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .remove(index.pack_new())?;

        Ok(())
    }
}

impl Delete<Address, Balance> for Storage {
    fn delete(&self, address: &Address) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .open_tree(TREE_ADDRESS_TO_BALANCE)?
            .remove(address.pack_new())?;

        Ok(())
    }
}

impl Delete<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    fn delete(
        &self,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unreferenced_message.as_ref());

        self.inner
            .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?
            .remove(key)?;

        Ok(())
    }
}

impl Delete<(MilestoneIndex, Receipt), ()> for Storage {
    fn delete(&self, (index, receipt): &(MilestoneIndex, Receipt)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        self.inner.open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?.remove(key)?;

        Ok(())
    }
}

impl Delete<(bool, TreasuryOutput), ()> for Storage {
    fn delete(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        self.inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?.remove(key)?;

        Ok(())
    }
}
