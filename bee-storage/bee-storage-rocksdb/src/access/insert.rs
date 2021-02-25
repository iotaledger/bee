// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*, system::System};

use bee_common::packable::Packable;
use bee_ledger::{
    balance::Balance,
    model::{OutputDiff, Receipt, TreasuryOutput, Unspent},
};
use bee_message::{
    address::{Address, Ed25519Address},
    ledger_index::LedgerIndex,
    milestone::{Milestone, MilestoneIndex},
    output::{ConsumedOutput, CreatedOutput, OutputId},
    payload::indexation::HashedIndex,
    solid_entry_point::SolidEntryPoint,
    Message, MessageId,
};
use bee_snapshot::info::SnapshotInfo;
use bee_storage::access::Insert;
use bee_tangle::{metadata::MessageMetadata, unconfirmed_message::UnconfirmedMessage};

#[async_trait::async_trait]
impl Insert<u8, System> for Storage {
    async fn insert(&self, key: &u8, value: &System) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self.inner.cf_handle(CF_SYSTEM).ok_or(Error::UnknownCf(CF_SYSTEM))?;

        self.inner.put_cf(&cf, [*key], value.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MessageId, Message> for Storage {
    async fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        self.inner.put_cf(&cf, message_id, message.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MessageId, MessageMetadata> for Storage {
    async fn insert(
        &self,
        message_id: &MessageId,
        metadata: &MessageMetadata,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        self.inner.put_cf(&cf, message_id, metadata.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(MessageId, MessageId), ()> for Storage {
    async fn insert(
        &self,
        (parent, child): &(MessageId, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        self.inner.put_cf(&cf, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(HashedIndex, MessageId), ()> for Storage {
    async fn insert(
        &self,
        (index, message_id): &(HashedIndex, MessageId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        self.inner.put_cf(&cf, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<OutputId, CreatedOutput> for Storage {
    async fn insert(
        &self,
        output_id: &OutputId,
        output: &CreatedOutput,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CREATED_OUTPUT))?;

        self.inner.put_cf(&cf, output_id.pack_new(), output.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<OutputId, ConsumedOutput> for Storage {
    async fn insert(
        &self,
        output_id: &OutputId,
        output: &ConsumedOutput,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT))?;

        self.inner.put_cf(&cf, output_id.pack_new(), output.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<Unspent, ()> for Storage {
    async fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_UNSPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_UNSPENT))?;

        self.inner.put_cf(&cf, unspent.pack_new(), [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(Ed25519Address, OutputId), ()> for Storage {
    async fn insert(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        self.inner.put_cf(&cf, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(), LedgerIndex> for Storage {
    async fn insert(&self, (): &(), index: &LedgerIndex) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        self.inner.put_cf(&cf, [0x00u8], index.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MilestoneIndex, Milestone> for Storage {
    async fn insert(
        &self,
        index: &MilestoneIndex,
        milestone: &Milestone,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_MILESTONE))?;

        self.inner.put_cf(&cf, index.pack_new(), milestone.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(), SnapshotInfo> for Storage {
    async fn insert(&self, (): &(), info: &SnapshotInfo) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        self.inner.put_cf(&cf, [0x00u8], info.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn insert(
        &self,
        sep: &SolidEntryPoint,
        index: &MilestoneIndex,
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        self.inner.put_cf(&cf, sep.pack_new(), index.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MilestoneIndex, OutputDiff> for Storage {
    async fn insert(&self, index: &MilestoneIndex, diff: &OutputDiff) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF))?;

        self.inner.put_cf(&cf, index.pack_new(), diff.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<Address, Balance> for Storage {
    async fn insert(&self, address: &Address, balance: &Balance) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ADDRESS_TO_BALANCE)
            .ok_or(Error::UnknownCf(CF_ADDRESS_TO_BALANCE))?;

        self.inner.put_cf(&cf, address.pack_new(), balance.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    async fn insert(
        &self,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE))?;

        let mut key = index.pack_new();
        key.extend_from_slice(unconfirmed_message.as_ref());

        self.inner.put_cf(&cf, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(MilestoneIndex, Receipt), ()> for Storage {
    async fn insert(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_RECEIPT))?;

        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        self.inner.put_cf(&cf, key, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<(bool, TreasuryOutput), ()> for Storage {
    async fn insert(
        &self,
        (spent, output): &(bool, TreasuryOutput),
        (): &(),
    ) -> Result<(), <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)
            .ok_or(Error::UnknownCf(CF_SPENT_TO_TREASURY_OUTPUT))?;

        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        self.inner.put_cf(&cf, key, [])?;

        Ok(())
    }
}
