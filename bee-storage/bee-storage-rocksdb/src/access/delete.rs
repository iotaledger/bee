// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

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
use bee_storage::access::Delete;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

#[async_trait::async_trait]
impl Delete<MessageId, Message> for Storage {
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MessageId, MessageMetadata> for Storage {
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?, message_id)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(MessageId, MessageId), ()> for Storage {
    async fn delete(&self, (parent, child): &(MessageId, MessageId)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner
            .delete_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(PaddedIndex, MessageId), ()> for Storage {
    async fn delete(
        &self,
        (index, message_id): &(PaddedIndex, MessageId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.delete_cf(self.cf_handle(CF_INDEX_TO_MESSAGE_ID)?, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<OutputId, CreatedOutput> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?, output_id.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<OutputId, ConsumedOutput> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?, output_id.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<Unspent, ()> for Storage {
    async fn delete(&self, unspent: &Unspent) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, unspent.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(Ed25519Address, OutputId), ()> for Storage {
    async fn delete(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        self.inner
            .delete_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(), LedgerIndex> for Storage {
    async fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.delete_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MilestoneIndex, Milestone> for Storage {
    async fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)?, index.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(), SnapshotInfo> for Storage {
    async fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner.delete_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn delete(&self, sep: &SolidEntryPoint) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?, sep.as_ref())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MilestoneIndex, OutputDiff> for Storage {
    async fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?, index.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<Address, Balance> for Storage {
    async fn delete(&self, address: &Address) -> Result<(), <Self as StorageBackend>::Error> {
        self.inner
            .delete_cf(self.cf_handle(CF_ADDRESS_TO_BALANCE)?, address.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(MilestoneIndex, UnreferencedMessage), ()> for Storage {
    async fn delete(
        &self,
        (index, unreferenced_message): &(MilestoneIndex, UnreferencedMessage),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unreferenced_message.as_ref());

        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE)?, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(MilestoneIndex, Receipt), ()> for Storage {
    async fn delete(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        self.inner
            .delete_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(bool, TreasuryOutput), ()> for Storage {
    async fn delete(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<(), <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        self.inner
            .delete_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, key)?;

        Ok(())
    }
}
