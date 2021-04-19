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
use bee_storage::access::Exist;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

#[async_trait::async_trait]
impl Exist<MessageId, Message> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        Ok(self.inner.get_cf(cf, message_id)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MessageId, MessageMetadata> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        Ok(self.inner.get_cf(cf, message_id)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(MessageId, MessageId), ()> for Storage {
    async fn exist(&self, (parent, child): &(MessageId, MessageId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self.inner.get_cf(cf, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(HashedIndex, MessageId), ()> for Storage {
    async fn exist(
        &self,
        (index, message_id): &(HashedIndex, MessageId),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        Ok(self.inner.get_cf(cf, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, CreatedOutput> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CREATED_OUTPUT))?;

        Ok(self.inner.get_cf(cf, output_id.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, ConsumedOutput> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT))?;

        Ok(self.inner.get_cf(cf, output_id.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<Unspent, ()> for Storage {
    async fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_UNSPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_UNSPENT))?;

        Ok(self.inner.get_cf(cf, unspent.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(Ed25519Address, OutputId), ()> for Storage {
    async fn exist(
        &self,
        (address, output_id): &(Ed25519Address, OutputId),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self.inner.get_cf(cf, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(), LedgerIndex> for Storage {
    async fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        Ok(self.inner.get_cf(cf, [0x00u8])?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MilestoneIndex, Milestone> for Storage {
    async fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_MILESTONE))?;

        Ok(self.inner.get_cf(cf, index.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(), SnapshotInfo> for Storage {
    async fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        Ok(self.inner.get_cf(cf, [0x00u8])?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn exist(&self, sep: &SolidEntryPoint) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        Ok(self.inner.get_cf(cf, sep.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MilestoneIndex, OutputDiff> for Storage {
    async fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF))?;

        Ok(self.inner.get_cf(cf, index.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<Address, Balance> for Storage {
    async fn exist(&self, address: &Address) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_ADDRESS_TO_BALANCE)
            .ok_or(Error::UnknownCf(CF_ADDRESS_TO_BALANCE))?;

        Ok(self.inner.get_cf(cf, address.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    async fn exist(
        &self,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE))?;

        let mut key = index.pack_new();
        key.extend_from_slice(unconfirmed_message.as_ref());

        Ok(self.inner.get_cf(cf, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(MilestoneIndex, Receipt), ()> for Storage {
    async fn exist(
        &self,
        (index, receipt): &(MilestoneIndex, Receipt),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)
            .ok_or(Error::UnknownCf(CF_MILESTONE_INDEX_TO_RECEIPT))?;

        let mut key = index.pack_new();
        key.extend_from_slice(&receipt.pack_new());

        Ok(self.inner.get_cf(cf, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(bool, TreasuryOutput), ()> for Storage {
    async fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)
            .ok_or(Error::UnknownCf(CF_SPENT_TO_TREASURY_OUTPUT))?;

        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        Ok(self.inner.get_cf(cf, key)?.is_some())
    }
}
