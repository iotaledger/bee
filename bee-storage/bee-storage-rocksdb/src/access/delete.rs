// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_common::packable::Packable;
use bee_ledger::{
    snapshot::info::SnapshotInfo,
    types::{Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent},
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::HashedIndex,
    Message, MessageId,
};
use bee_storage::access::Delete;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

#[async_trait::async_trait]
impl Delete<MessageId, Message> for Storage {
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        self.inner.delete_cf(&cf, message_id)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MessageId, MessageMetadata> for Storage {
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        self.inner.delete_cf(&cf, message_id)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(MessageId, MessageId), ()> for Storage {
    async fn delete(&self, (parent, child): &(MessageId, MessageId)) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner.delete_cf(&cf, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(HashedIndex, MessageId), ()> for Storage {
    async fn delete(
        &self,
        (index, message_id): &(HashedIndex, MessageId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.delete_cf(&cf, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<OutputId, CreatedOutput> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CREATED_OUTPUT))?;

        self.inner.delete_cf(&cf, output_id.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<OutputId, ConsumedOutput> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT))?;

        self.inner.delete_cf(&cf, output_id.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<Unspent, ()> for Storage {
    async fn delete(&self, unspent: &Unspent) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_UNSPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_UNSPENT))?;

        self.inner.delete_cf(&cf, unspent.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(Ed25519Address, OutputId), ()> for Storage {
    async fn delete(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        self.inner.delete_cf(&cf, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(), LedgerIndex> for Storage {
    async fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        self.inner.delete_cf(&cf, [0x00u8])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MilestoneIndex, Milestone> for Storage {
    async fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_MILESTONE))?;

        self.inner.delete_cf(&cf, index.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(), SnapshotInfo> for Storage {
    async fn delete(&self, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        self.inner.delete_cf(&cf, [0x00u8])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn delete(&self, sep: &SolidEntryPoint) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        self.inner.delete_cf(&cf, sep.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MilestoneIndex, OutputDiff> for Storage {
    async fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF))?;

        self.inner.delete_cf(&cf, index.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<Address, Balance> for Storage {
    async fn delete(&self, address: &Address) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ADDRESS_TO_BALANCE)
            .ok_or(Error::UnknownCf(CF_ADDRESS_TO_BALANCE))?;

        self.inner.delete_cf(&cf, address.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    async fn delete(
        &self,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE))?;

        let mut key = index.pack_new();
        key.extend_from_slice(unconfirmed_message.as_ref());

        self.inner.delete_cf(&cf, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(MilestoneIndex, Receipt), ()> for Storage {
    async fn delete(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_RECEIPT))?;

        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        self.inner.delete_cf(&cf, key)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<(bool, TreasuryOutput), ()> for Storage {
    async fn delete(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)
            .ok_or(Error::UnknownCf(CF_SPENT_TO_TREASURY_OUTPUT))?;

        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        self.inner.delete_cf(&cf, key)?;

        Ok(())
    }
}
