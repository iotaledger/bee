// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
    system::System,
};

use bee_common::packable::Packable;
use bee_ledger::{
    snapshot::info::SnapshotInfo,
    types::{Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent},
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::{PaddedIndex, INDEXATION_PADDED_INDEX_LENGTH},
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_storage::access::AsStream;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage,
};

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

            async fn stream(&'a self) -> Result<Self::Stream, <Self as StorageBackend>::Error> {
                Ok(StorageStream::new(
                    self.inner.iterator_cf(self.cf_handle($cf)?, IteratorMode::Start),
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

impl<'a> StorageStream<'a, u8, System> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (u8, System) {
        (
            // Unpacking from storage is fine.
            u8::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            System::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, MessageId, Message> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, Message) {
        (
            // Unpacking from storage is fine.
            MessageId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Message::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, MessageId, MessageMetadata> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, MessageMetadata) {
        (
            // Unpacking from storage is fine.
            MessageId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MessageMetadata::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, (MessageId, MessageId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MessageId, MessageId), ()) {
        let (mut parent, mut child) = key.split_at(MESSAGE_ID_LENGTH);

        (
            (
                // Unpacking from storage is fine.
                MessageId::unpack_unchecked(&mut parent).unwrap(),
                // Unpacking from storage is fine.
                MessageId::unpack_unchecked(&mut child).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageStream<'a, (PaddedIndex, MessageId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((PaddedIndex, MessageId), ()) {
        let (index, mut message_id) = key.split_at(INDEXATION_PADDED_INDEX_LENGTH);
        // Unpacking from storage is fine.
        let index: [u8; INDEXATION_PADDED_INDEX_LENGTH] = index.try_into().unwrap();

        (
            // Unpacking from storage is fine.
            (
                PaddedIndex::new(index),
                MessageId::unpack_unchecked(&mut message_id).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageStream<'a, OutputId, CreatedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, CreatedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            CreatedOutput::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, OutputId, ConsumedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, ConsumedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            ConsumedOutput::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, Unspent, ()> {
    fn unpack_key_value(mut key: &[u8], _: &[u8]) -> (Unspent, ()) {
        (
            // Unpacking from storage is fine.
            Unspent::unpack_unchecked(&mut key).unwrap(),
            (),
        )
    }
}

impl<'a> StorageStream<'a, (Ed25519Address, OutputId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((Ed25519Address, OutputId), ()) {
        let (mut address, mut output_id) = key.split_at(MESSAGE_ID_LENGTH);

        (
            (
                // Unpacking from storage is fine.
                Ed25519Address::unpack_unchecked(&mut address).unwrap(),
                // Unpacking from storage is fine.
                OutputId::unpack_unchecked(&mut output_id).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageStream<'a, (), LedgerIndex> {
    fn unpack_key_value(_: &[u8], mut value: &[u8]) -> ((), LedgerIndex) {
        (
            (),
            // Unpacking from storage is fine.
            LedgerIndex::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, MilestoneIndex, Milestone> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, Milestone) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Milestone::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, (), SnapshotInfo> {
    fn unpack_key_value(_: &[u8], mut value: &[u8]) -> ((), SnapshotInfo) {
        (
            (),
            // Unpacking from storage is fine.
            SnapshotInfo::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, SolidEntryPoint, MilestoneIndex> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (SolidEntryPoint, MilestoneIndex) {
        (
            // Unpacking from storage is fine.
            SolidEntryPoint::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, MilestoneIndex, OutputDiff> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, OutputDiff) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            OutputDiff::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, Address, Balance> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (Address, Balance) {
        (
            // Unpacking from storage is fine.
            Address::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Balance::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageStream<'a, (MilestoneIndex, UnconfirmedMessage), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MilestoneIndex, UnconfirmedMessage), ()) {
        let (mut index, mut unconfirmed_message) = key.split_at(std::mem::size_of::<MilestoneIndex>());

        (
            (
                // Unpacking from storage is fine.
                MilestoneIndex::unpack_unchecked(&mut index).unwrap(),
                // Unpacking from storage is fine.
                UnconfirmedMessage::unpack_unchecked(&mut unconfirmed_message).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageStream<'a, (MilestoneIndex, Receipt), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MilestoneIndex, Receipt), ()) {
        let (mut index, mut receipt) = key.split_at(std::mem::size_of::<MilestoneIndex>());

        (
            (
                // Unpacking from storage is fine.
                MilestoneIndex::unpack_unchecked(&mut index).unwrap(),
                // Unpacking from storage is fine.
                Receipt::unpack_unchecked(&mut receipt).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageStream<'a, (bool, TreasuryOutput), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((bool, TreasuryOutput), ()) {
        let (mut index, mut receipt) = key.split_at(std::mem::size_of::<bool>());

        (
            (
                // Unpacking from storage is fine.
                bool::unpack_unchecked(&mut index).unwrap(),
                // Unpacking from storage is fine.
                TreasuryOutput::unpack_unchecked(&mut receipt).unwrap(),
            ),
            (),
        )
    }
}

impl_stream!(u8, System, CF_SYSTEM);
impl_stream!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
impl_stream!(MessageId, MessageMetadata, CF_MESSAGE_ID_TO_METADATA);
impl_stream!((MessageId, MessageId), (), CF_MESSAGE_ID_TO_MESSAGE_ID);
impl_stream!((PaddedIndex, MessageId), (), CF_INDEX_TO_MESSAGE_ID);
impl_stream!(OutputId, CreatedOutput, CF_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_stream!(OutputId, ConsumedOutput, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_stream!(Unspent, (), CF_OUTPUT_ID_UNSPENT);
impl_stream!((Ed25519Address, OutputId), (), CF_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_stream!((), LedgerIndex, CF_LEDGER_INDEX);
impl_stream!(MilestoneIndex, Milestone, CF_MILESTONE_INDEX_TO_MILESTONE);
impl_stream!((), SnapshotInfo, CF_SNAPSHOT_INFO);
impl_stream!(SolidEntryPoint, MilestoneIndex, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX);
impl_stream!(MilestoneIndex, OutputDiff, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_stream!(Address, Balance, CF_ADDRESS_TO_BALANCE);
impl_stream!(
    (MilestoneIndex, UnconfirmedMessage),
    (),
    CF_MILESTONE_INDEX_TO_UNCONFIRMED_MESSAGE
);
impl_stream!((MilestoneIndex, Receipt), (), CF_MILESTONE_INDEX_TO_RECEIPT);
impl_stream!((bool, TreasuryOutput), (), CF_SPENT_TO_TREASURY_OUTPUT);
