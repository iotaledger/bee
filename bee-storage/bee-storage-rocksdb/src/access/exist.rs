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
use bee_storage::access::Exist;

#[async_trait::async_trait]
impl Exist<MessageId, Message> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        Ok(self.inner.get_cf(&cf, message_id)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<MessageId, MessageMetadata> for Storage {
    async fn exist(&self, message_id: &MessageId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        Ok(self.inner.get_cf(&cf, message_id)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(MessageId, MessageId), ()> for Storage {
    async fn exist(&self, (parent, child): &(MessageId, MessageId)) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        let mut key = parent.as_ref().to_vec();
        key.extend_from_slice(child.as_ref());

        Ok(self.inner.get_cf(&cf, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(HashedIndex, MessageId), ()> for Storage {
    async fn exist(&self, (index, message_id): &(HashedIndex, MessageId)) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        let mut key = index.as_ref().to_vec();
        key.extend_from_slice(message_id.as_ref());

        Ok(self.inner.get_cf(&cf, key)?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, Output> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_OUTPUT))?;

        Ok(self.inner.get_cf(&cf, output_id.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<OutputId, Spent> for Storage {
    async fn exist(&self, output_id: &OutputId) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_SPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_SPENT))?;

        Ok(self.inner.get_cf(&cf, output_id.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<Unspent, ()> for Storage {
    async fn exist(&self, unspent: &Unspent) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_UNSPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_UNSPENT))?;

        Ok(self.inner.get_cf(&cf, unspent.pack_new())?.is_some())
    }
}

#[async_trait::async_trait]
impl Exist<(Ed25519Address, OutputId), ()> for Storage {
    async fn exist(&self, (address, output_id): &(Ed25519Address, OutputId)) -> Result<bool, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        let mut key = address.as_ref().to_vec();
        key.extend_from_slice(&output_id.pack_new());

        Ok(self.inner.get_cf(&cf, key)?.is_some())
    }
}
