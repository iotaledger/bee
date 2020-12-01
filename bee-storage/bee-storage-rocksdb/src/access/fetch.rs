// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_common::packable::Packable;
use bee_ledger::{output::Output, spent::Spent};
use bee_message::{
    payload::{
        indexation::{HashedIndex, HASHED_INDEX_LENGTH},
        transaction::{Ed25519Address, OutputId, ED25519_ADDRESS_LENGTH, OUTPUT_ID_LENGTH},
    },
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_protocol::tangle::MessageMetadata;
use bee_storage::access::Fetch;

use std::convert::TryInto;

#[async_trait::async_trait]
impl Fetch<MessageId, Message> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<Message>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE))?;

        if let Some(res) = self.inner.get_cf(&cf, message_id)? {
            Ok(Some(Message::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, MessageMetadata> for Storage {
    async fn fetch(&self, message_id: &MessageId) -> Result<Option<MessageMetadata>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_METADATA)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_METADATA))?;

        if let Some(res) = self.inner.get_cf(&cf, message_id)? {
            Ok(Some(MessageMetadata::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<MessageId, Vec<MessageId>> for Storage {
    async fn fetch(&self, parent: &MessageId) -> Result<Option<Vec<MessageId>>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_MESSAGE_ID_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_MESSAGE_ID_TO_MESSAGE_ID))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf, parent)
                .map(|(key, _)| {
                    let (_, child) = key.split_at(MESSAGE_ID_LENGTH);
                    let child: [u8; MESSAGE_ID_LENGTH] = child.try_into().unwrap();
                    MessageId::from(child)
                })
                .take(self.config.fetch_edge_limit)
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<HashedIndex, Vec<MessageId>> for Storage {
    async fn fetch(&self, index: &HashedIndex) -> Result<Option<Vec<MessageId>>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_INDEX_TO_MESSAGE_ID)
            .ok_or(Error::UnknownCf(CF_INDEX_TO_MESSAGE_ID))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf, index)
                .map(|(key, _)| {
                    let (_, message_id) = key.split_at(HASHED_INDEX_LENGTH);
                    let message_id: [u8; MESSAGE_ID_LENGTH] = message_id.try_into().unwrap();
                    MessageId::from(message_id)
                })
                .take(self.config.fetch_index_limit)
                .collect(),
        ))
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, Output> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<Output>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_OUTPUT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_OUTPUT))?;

        if let Some(res) = self.inner.get_cf(&cf, output_id.pack_new())? {
            Ok(Some(Output::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<OutputId, Spent> for Storage {
    async fn fetch(&self, output_id: &OutputId) -> Result<Option<Spent>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_OUTPUT_ID_TO_SPENT)
            .ok_or(Error::UnknownCf(CF_OUTPUT_ID_TO_SPENT))?;

        if let Some(res) = self.inner.get_cf(&cf, output_id.pack_new())? {
            Ok(Some(Spent::unpack(&mut res.as_slice()).unwrap()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Fetch<Ed25519Address, Vec<OutputId>> for Storage {
    async fn fetch(&self, address: &Ed25519Address) -> Result<Option<Vec<OutputId>>, <Self as Backend>::Error>
    where
        Self: Sized,
    {
        let cf = self
            .inner
            .cf_handle(CF_ED25519_ADDRESS_TO_OUTPUT_ID)
            .ok_or(Error::UnknownCf(CF_ED25519_ADDRESS_TO_OUTPUT_ID))?;

        Ok(Some(
            self.inner
                .prefix_iterator_cf(&cf, address)
                .map(|(key, _)| {
                    let (_, output_id) = key.split_at(ED25519_ADDRESS_LENGTH);
                    From::<[u8; OUTPUT_ID_LENGTH]>::from(output_id.try_into().unwrap())
                })
                .take(self.config.fetch_output_id_limit)
                .collect(),
        ))
    }
}
