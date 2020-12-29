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
use bee_storage::access::Delete;

#[async_trait::async_trait]
impl Delete<MessageId, Message> for Storage {
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as Backend>::Error> {
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
    async fn delete(&self, message_id: &MessageId) -> Result<(), <Self as Backend>::Error> {
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
    async fn delete(&self, (parent, child): &(MessageId, MessageId)) -> Result<(), <Self as Backend>::Error> {
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
    async fn delete(&self, (index, message_id): &(HashedIndex, MessageId)) -> Result<(), <Self as Backend>::Error> {
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
impl Delete<OutputId, Output> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_OUTPUT))?;

        self.inner.delete_cf(&cf, output_id.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<OutputId, Spent> for Storage {
    async fn delete(&self, output_id: &OutputId) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_SPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_SPENT))?;

        self.inner.delete_cf(&cf, output_id.pack_new())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<Unspent, ()> for Storage {
    async fn delete(&self, unspent: &Unspent) -> Result<(), <Self as Backend>::Error> {
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
    async fn delete(&self, (address, output_id): &(Ed25519Address, OutputId)) -> Result<(), <Self as Backend>::Error> {
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
    async fn delete(&self, (): &()) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_LEDGER_INDEX)
            .ok_or(Error::UnknownCf(CF_LEDGER_INDEX))?;

        self.inner.delete_cf(&cf, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<MilestoneIndex, Milestone> for Storage {
    async fn delete(&self, index: &MilestoneIndex) -> Result<(), <Self as Backend>::Error> {
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
    async fn delete(&self, (): &()) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SNAPSHOT_INFO)
            .ok_or(Error::UnknownCf(CF_SNAPSHOT_INFO))?;

        self.inner.delete_cf(&cf, [])?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Delete<SolidEntryPoint, MilestoneIndex> for Storage {
    async fn delete(&self, sep: &SolidEntryPoint) -> Result<(), <Self as Backend>::Error> {
        let cf = self
            .inner
            .cf_handle(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX)
            .ok_or(Error::UnknownCf(CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX))?;

        self.inner.delete_cf(&cf, sep.pack_new())?;

        Ok(())
    }
}
