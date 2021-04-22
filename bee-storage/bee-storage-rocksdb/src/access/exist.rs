// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::storage::*;

use bee_common::packable::Packable;
use bee_ledger::{
    snapshot::info::SnapshotInfo,
    types::{Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent},
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::PaddedIndex,
    Message, MessageId,
};
use bee_storage::access::Exist;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

#[async_trait::async_trait]
impl Exist<MessageId, Message> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE)?, message_id)?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MessageId, MessageMetadata> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?, message_id)?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(MessageId, MessageId), ()> for Storage {
    async fn exist(&self, (parent, child): &(MessageId, MessageId)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)?, key)?
            .is_some())
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

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_INDEX_TO_MESSAGE_ID)?, key)?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, CreatedOutput> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_OUTPUT_ID_TO_CREATED_OUTPUT)?, output_id.pack_new())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, ConsumedOutput> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_OUTPUT_ID_TO_CONSUMED_OUTPUT)?, output_id.pack_new())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<Unspent, ()> for Storage {
    async fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_OUTPUT_ID_UNSPENT)?, unspent.pack_new())?
            .is_some())
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
            .get_cf(self.cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)?, key)?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(), LedgerIndex> for Storage {
    async fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self.inner.get_cf(self.cf_handle(CF_LEDGER_INDEX)?, [0x00u8])?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MilestoneIndex, Milestone> for Storage {
    async fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_MILESTONE)?, index.pack_new())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(), SnapshotInfo> for Storage {
    async fn exist(&self, (): &()) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_SNAPSHOT_INFO)?, [0x00u8])?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn exist(&self, sep: &SolidEntryPoint) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)?, sep.pack_new())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MilestoneIndex, OutputDiff> for Storage {
    async fn exist(&self, index: &MilestoneIndex) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_OUTPUT_DIFF)?, index.pack_new())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<Address, Balance> for Storage {
    async fn exist(&self, address: &Address) -> Result<bool, <Self as StorageBackend>::Error> {
        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_ADDRESS_TO_BALANCE)?, address.pack_new())?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(MilestoneIndex, UnconfirmedMessage), ()> for Storage {
    async fn exist(
        &self,
        (index, unconfirmed_message): &(MilestoneIndex, UnconfirmedMessage),
    ) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = index.pack_new();
        key.extend_from_slice(unconfirmed_message.as_ref());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE)?, key)?
            .is_some())
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
            .get_cf(self.cf_handle(CF_MILESTONE_INDEX_TO_RECEIPT)?, key)?
            .is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(bool, TreasuryOutput), ()> for Storage {
    async fn exist(&self, (spent, output): &(bool, TreasuryOutput)) -> Result<bool, <Self as StorageBackend>::Error> {
        let mut key = spent.pack_new();
        key.extend_from_slice(&output.pack_new());

        Ok(self
            .inner
            .get_cf(self.cf_handle(CF_SPENT_TO_TREASURY_OUTPUT)?, key)?
            .is_some())
    }
}
