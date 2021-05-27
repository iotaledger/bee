// Copyright 2021 IOTA Stiftung
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
use bee_storage::{access::Exist, backend::StorageBackend};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

#[async_trait::async_trait]
impl Exist<MessageId, Message> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_MESSAGE)?
            .contains_key(message_id)?)
    }
}

#[async_trait::async_trait]
impl Exist<MessageId, MessageMetadata> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MESSAGE_ID_TO_METADATA)?
            .contains_key(message_id)?)
    }
}

#[async_trait::async_trait]
impl Exist<(MessageId, MessageId), ()> for Storage {
    async fn exist(&self, (parent, child): &(MessageId, MessageId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self.inner.open_tree(TREE_MESSAGE_ID_TO_MESSAGE_ID)?.contains_key(key)?)
    }
}

#[async_trait::async_trait]
impl Exist<(PaddedIndex, MessageId), ()> for Storage {
    async fn exist(
        &self,
        (index, message_id): &(PaddedIndex, MessageId),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        Ok(self.inner.open_tree(TREE_INDEX_TO_MESSAGE_ID)?.contains_key(key)?)
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, CreatedOutput> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CREATED_OUTPUT)?
            .contains_key(output_id.pack_new())?)
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, ConsumedOutput> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT)?
            .contains_key(output_id.pack_new())?)
    }
}

#[async_trait::async_trait]
impl Exist<Unspent, ()> for Storage {
    async fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_OUTPUT_ID_UNSPENT)?
            .contains_key(unspent.pack_new())?)
    }
}

#[async_trait::async_trait]
impl Exist<(Ed25519Address, OutputId), ()> for Storage {
    async fn exist(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self
            .inner
            .open_tree(TREE_ED25519_ADDRESS_TO_OUTPUT_ID)?
            .contains_key(key)?)
    }
}

#[async_trait::async_trait]
impl Exist<(), LedgerIndex> for Storage {
    async fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.open_tree(TREE_LEDGER_INDEX)?.contains_key([0x00u8])?)
    }
}

#[async_trait::async_trait]
impl Exist<MilestoneIndex, Milestone> for Storage {
    async fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_MILESTONE)?
            .contains_key(index.pack_new())?)
    }
}

#[async_trait::async_trait]
impl Exist<(), SnapshotInfo> for Storage {
    async fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.open_tree(TREE_SNAPSHOT_INFO)?.contains_key([0x00u8])?)
    }
}

#[async_trait::async_trait]
impl Exist<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn exist(&self, sep: &SolidEntryPoint) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?
            .contains_key(sep.pack_new())?)
    }
}

#[async_trait::async_trait]
impl Exist<MilestoneIndex, OutputDiff> for Storage {
    async fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF)?
            .contains_key(index.pack_new())?)
    }
}

#[async_trait::async_trait]
impl Exist<Address, Balance> for Storage {
    async fn exist(&self, address: &Address) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .open_tree(TREE_ADDRESS_TO_BALANCE)?
            .contains_key(address.pack_new())?)
    }
}

#[async_trait::async_trait]
impl Exist<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    async fn exist(
        &self,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unreferenced_message.as_ref());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?
            .contains_key(key)?)
    }
}

#[async_trait::async_trait]
impl Exist<(MilestoneIndex, Receipt), ()> for Storage {
    async fn exist(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        Ok(self
            .inner
            .open_tree(TREE_MILESTONE_INDEX_TO_RECEIPT)?
            .contains_key(key)?)
    }
}

#[async_trait::async_trait]
impl Exist<(bool, TreasuryOutput), ()> for Storage {
    async fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        Ok(self.inner.open_tree(TREE_SPENT_TO_TREASURY_OUTPUT)?.contains_key(key)?)
    }
}
