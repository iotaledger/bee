// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_common::packable::Packable;
use bee_ledger::{output::Output, spent::Spent, unspent::Unspent};
use bee_message::{
    payload::{
        indexation::HashedIndex,
        transaction::{Ed25519Address, OutputId},
    },
    Message, MessageId,
};
use bee_protocol::tangle::MessageMetadata;
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
