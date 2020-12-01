// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, storage::*};

use bee_common::packable::Packable;
use bee_ledger::{output::Output, spent::Spent, unspent::Unspent};
use bee_message::{
    payload::{
        indexation::{HashedIndex, HASHED_INDEX_LENGTH},
        transaction::{Ed25519Address, OutputId},
    },
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_protocol::tangle::MessageMetadata;
use bee_storage::access::AsStream;

use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use pin_project::pin_project;
use rocksdb::{DBIterator, IteratorMode};

use std::{convert::TryInto, marker::PhantomData, pin::Pin};

#[pin_project(project = StorageStreamProj)]
pub struct StorageStream<'a, K, V> {
    #[pin]
    inner: DBIterator<'a>,
    budget: usize,
    counter: usize,
    marker: PhantomData<(K, V)>,
}

impl<'a, K, V> StorageStream<'a, K, V> {
    fn new(inner: DBIterator<'a>, budget: usize) -> Self {
        StorageStream::<K, V> {
            inner,
            budget,
            counter: 0,
            marker: PhantomData,
        }
    }
}

macro_rules! impl_stream {
    ($key:ty, $value:ty, $cf:expr) => {
        #[async_trait::async_trait]
        impl<'a> AsStream<'a, $key, $value> for Storage {
            type Stream = StorageStream<'a, $key, $value>;

            async fn stream(&'a self) -> Result<Self::Stream, <Self as Backend>::Error>
            where
                Self: Sized,
            {
                let cf = self.inner.cf_handle($cf).ok_or(Error::UnknownCf($cf))?;

                Ok(StorageStream::new(
                    self.inner.iterator_cf(cf, IteratorMode::Start),
                    self.config.iteration_budget,
                ))
            }
        }

        impl<'a> Stream for StorageStream<'a, $key, $value> {
            type Item = ($key, $value);

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
                let StorageStreamProj {
                    mut inner,
                    budget,
                    counter,
                    ..
                } = self.project();

                if counter == budget {
                    *counter = 0;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }

                *counter += 1;

                let item = inner
                    .next()
                    .map(|(key, value)| Self::unpack_key_value(&key, &value));

                if inner.valid() {
                    Poll::Ready(item)
                } else {
                    Poll::Ready(None)
                }
            }
        }
    };
}

impl<'a> StorageStream<'a, MessageId, Message> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, Message) {
        (
            MessageId::unpack(&mut key).unwrap(),
            Message::unpack(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, MessageId, MessageMetadata> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, MessageMetadata) {
        (
            MessageId::unpack(&mut key).unwrap(),
            MessageMetadata::unpack(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, (MessageId, MessageId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MessageId, MessageId), ()) {
        let (mut parent, mut child) = key.split_at(MESSAGE_ID_LENGTH);

        (
            (
                MessageId::unpack(&mut parent).unwrap(),
                MessageId::unpack(&mut child).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageStream<'a, (HashedIndex, MessageId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((HashedIndex, MessageId), ()) {
        let (index, mut message_id) = key.split_at(HASHED_INDEX_LENGTH);
        // TODO review when we have fixed size index
        let index: [u8; HASHED_INDEX_LENGTH] = index.try_into().unwrap();

        (
            (HashedIndex::new(index), MessageId::unpack(&mut message_id).unwrap()),
            (),
        )
    }
}

impl<'a> StorageStream<'a, OutputId, Output> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, Output) {
        (OutputId::unpack(&mut key).unwrap(), Output::unpack(&mut value).unwrap())
    }
}

impl<'a> StorageStream<'a, OutputId, Spent> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, Spent) {
        (OutputId::unpack(&mut key).unwrap(), Spent::unpack(&mut value).unwrap())
    }
}

impl<'a> StorageStream<'a, Unspent, ()> {
    fn unpack_key_value(mut key: &[u8], _: &[u8]) -> (Unspent, ()) {
        (Unspent::unpack(&mut key).unwrap(), ())
    }
}

impl<'a> StorageStream<'a, (Ed25519Address, OutputId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((Ed25519Address, OutputId), ()) {
        let (mut address, mut output_id) = key.split_at(MESSAGE_ID_LENGTH);

        (
            (
                Ed25519Address::unpack(&mut address).unwrap(),
                OutputId::unpack(&mut output_id).unwrap(),
            ),
            (),
        )
    }
}

impl_stream!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
impl_stream!(MessageId, MessageMetadata, CF_MESSAGE_ID_TO_METADATA);
impl_stream!((MessageId, MessageId), (), CF_MESSAGE_ID_TO_MESSAGE_ID);
impl_stream!((HashedIndex, MessageId), (), CF_INDEX_TO_MESSAGE_ID);
impl_stream!(OutputId, Output, CF_OUTPUT_ID_TO_OUTPUT);
impl_stream!(OutputId, Spent, CF_OUTPUT_ID_TO_SPENT);
impl_stream!(Unspent, (), CF_OUTPUT_ID_UNSPENT);
impl_stream!((Ed25519Address, OutputId), (), CF_ED25519_ADDRESS_TO_OUTPUT_ID);
