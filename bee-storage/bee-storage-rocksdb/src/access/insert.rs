// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_common::packable::Packable;
use bee_ledger::model::{LedgerIndex, Output, Spent, Unspent};
use bee_message::{
    payload::{
        indexation::HashedIndex,
        transaction::{Ed25519Address, OutputId},
    },
    Message, MessageId,
};
use bee_protocol::{
    tangle::{MessageMetadata, SolidEntryPoint},
    Milestone, MilestoneIndex,
};
use bee_snapshot::info::SnapshotInfo;
use bee_storage::access::Insert;

#[async_trait::async_trait]
impl Insert<MessageId, Message> for Storage {
    async fn insert(&self, message_id: &MessageId, message: &Message) -> Result<(), <Self as Backend>::Error> {
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
    async fn insert(&self, message_id: &MessageId, metadata: &MessageMetadata) -> Result<(), <Self as Backend>::Error> {
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
    async fn insert(&self, (parent, child): &(MessageId, MessageId), (): &()) -> Result<(), <Self as Backend>::Error> {
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
    ) -> Result<(), <Self as Backend>::Error> {
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
impl Insert<OutputId, Output> for Storage {
    async fn insert(&self, output_id: &OutputId, output: &Output) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_OUTPUT))?;

        self.inner.put_cf(&cf, output_id.pack_new(), output.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<OutputId, Spent> for Storage {
    async fn insert(&self, output_id: &OutputId, spent: &Spent) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_SPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_SPENT))?;

        self.inner.put_cf(&cf, output_id.pack_new(), spent.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<Unspent, ()> for Storage {
    async fn insert(&self, unspent: &Unspent, (): &()) -> Result<(), <Self as Backend>::Error> {
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
    ) -> Result<(), <Self as Backend>::Error> {
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
    async fn insert(&self, (): &(), index: &LedgerIndex) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        self.inner.put_cf(&cf, [], index.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<MilestoneIndex, Milestone> for Storage {
    async fn insert(&self, index: &MilestoneIndex, milestone: &Milestone) -> Result<(), <Self as Backend>::Error> {
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
    async fn insert(&self, (): &(), info: &SnapshotInfo) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        self.inner.put_cf(&cf, [], info.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Insert<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn insert(&self, sep: &SolidEntryPoint, index: &MilestoneIndex) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        self.inner.put_cf(&cf, sep.pack_new(), index.pack_new())?;

        Ok(())
    }
}
