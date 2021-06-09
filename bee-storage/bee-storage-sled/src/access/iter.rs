// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Stream access operations.

use crate::{storage::Storage, trees::*};

use bee_common::packable::Packable;
use bee_ledger::types::{
    snapshot::SnapshotInfo, Balance, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput,
    Unspent,
};
use bee_message::{
    address::{Address, Ed25519Address},
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::indexation::{PaddedIndex, INDEXATION_PADDED_INDEX_LENGTH},
    Message, MessageId, MESSAGE_ID_LENGTH,
};
use bee_storage::{access::AsIterator, backend::StorageBackend, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};

use std::{convert::TryInto, marker::PhantomData};

/// Type used to stream a subtree.
pub struct StorageIterator<'a, K, V> {
    inner: sled::Iter,
    marker: PhantomData<&'a (K, V)>,
}

impl<'a, K, V> StorageIterator<'a, K, V> {
    fn new(inner: sled::Iter) -> Self {
        StorageIterator::<K, V> {
            inner,
            marker: PhantomData,
        }
    }
}

macro_rules! impl_stream {
    ($key:ty, $value:ty, $cf:expr) => {
        impl<'a> AsIterator<'a, $key, $value> for Storage {
            type AsIter = StorageIterator<'a, $key, $value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                Ok(StorageIterator::new(self.inner.open_tree($cf)?.iter()))
            }
        }

        /// A stream to iterate over all key-value pairs of a column family.
        impl<'a> Iterator for StorageIterator<'a, $key, $value> {
            type Item = Result<($key, $value), <Storage as StorageBackend>::Error>;

            fn next(&mut self) -> Option<Self::Item> {
                self.inner.next().map(|result| {
                    result
                        .map(|(key, value)| Self::unpack_key_value(&key, &value))
                        .map_err(From::from)
                })

                // inner.status()?;
                //
                // if inner.valid() {
                //     Poll::Ready(item)
                // } else {
                //     Poll::Ready(None)
                // }
            }
        }
    };
}

impl<'a> StorageIterator<'a, u8, System> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (u8, System) {
        (
            // Unpacking from storage is fine.
            u8::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            System::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MessageId, Message> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, Message) {
        (
            // Unpacking from storage is fine.
            MessageId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Message::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MessageId, MessageMetadata> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, MessageMetadata) {
        (
            // Unpacking from storage is fine.
            MessageId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MessageMetadata::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (MessageId, MessageId), ()> {
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

impl<'a> StorageIterator<'a, (PaddedIndex, MessageId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((PaddedIndex, MessageId), ()) {
        let (index, mut message_id) = key.split_at(INDEXATION_PADDED_INDEX_LENGTH);
        // Unpacking from storage is fine.
        let index: [u8; INDEXATION_PADDED_INDEX_LENGTH] = index.try_into().unwrap();

        (
            (
                PaddedIndex::new(index),
                // Unpacking from storage is fine.
                MessageId::unpack_unchecked(&mut message_id).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageIterator<'a, OutputId, CreatedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, CreatedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            CreatedOutput::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, OutputId, ConsumedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, ConsumedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            ConsumedOutput::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, Unspent, ()> {
    fn unpack_key_value(mut key: &[u8], _: &[u8]) -> (Unspent, ()) {
        (
            // Unpacking from storage is fine.
            Unspent::unpack_unchecked(&mut key).unwrap(),
            (),
        )
    }
}

impl<'a> StorageIterator<'a, (Ed25519Address, OutputId), ()> {
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

impl<'a> StorageIterator<'a, (), LedgerIndex> {
    fn unpack_key_value(_: &[u8], mut value: &[u8]) -> ((), LedgerIndex) {
        (
            (),
            // Unpacking from storage is fine.
            LedgerIndex::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneIndex, Milestone> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, Milestone) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Milestone::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (), SnapshotInfo> {
    fn unpack_key_value(_: &[u8], mut value: &[u8]) -> ((), SnapshotInfo) {
        (
            (),
            // Unpacking from storage is fine.
            SnapshotInfo::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, SolidEntryPoint, MilestoneIndex> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (SolidEntryPoint, MilestoneIndex) {
        (
            // Unpacking from storage is fine.
            SolidEntryPoint::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneIndex, OutputDiff> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, OutputDiff) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            OutputDiff::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, Address, Balance> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (Address, Balance) {
        (
            // Unpacking from storage is fine.
            Address::unpack_unchecked(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Balance::unpack_unchecked(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (MilestoneIndex, UnreferencedMessage), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MilestoneIndex, UnreferencedMessage), ()) {
        let (mut index, mut unreferenced_message) = key.split_at(std::mem::size_of::<MilestoneIndex>());

        (
            (
                // Unpacking from storage is fine.
                MilestoneIndex::unpack_unchecked(&mut index).unwrap(),
                // Unpacking from storage is fine.
                UnreferencedMessage::unpack_unchecked(&mut unreferenced_message).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageIterator<'a, (MilestoneIndex, Receipt), ()> {
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

impl<'a> StorageIterator<'a, (bool, TreasuryOutput), ()> {
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

impl<'a> AsIterator<'a, u8, System> for Storage {
    type AsIter = StorageIterator<'a, u8, System>;

    fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
        Ok(StorageIterator::new(self.inner.iter()))
    }
}

/// A stream to iterate over all key-value pairs of a column family.
impl<'a> Iterator for StorageIterator<'a, u8, System> {
    type Item = Result<(u8, System), <Storage as StorageBackend>::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result
                .map(|(key, value)| Self::unpack_key_value(&key, &value))
                .map_err(From::from)
        })

        // Poll::Ready(item)
    }
}

impl_stream!(MessageId, Message, TREE_MESSAGE_ID_TO_MESSAGE);
impl_stream!(MessageId, MessageMetadata, TREE_MESSAGE_ID_TO_METADATA);
impl_stream!((MessageId, MessageId), (), TREE_MESSAGE_ID_TO_MESSAGE_ID);
impl_stream!((PaddedIndex, MessageId), (), TREE_INDEX_TO_MESSAGE_ID);
impl_stream!(OutputId, CreatedOutput, TREE_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_stream!(OutputId, ConsumedOutput, TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_stream!(Unspent, (), TREE_OUTPUT_ID_UNSPENT);
impl_stream!((Ed25519Address, OutputId), (), TREE_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_stream!((), LedgerIndex, TREE_LEDGER_INDEX);
impl_stream!(MilestoneIndex, Milestone, TREE_MILESTONE_INDEX_TO_MILESTONE);
impl_stream!((), SnapshotInfo, TREE_SNAPSHOT_INFO);
impl_stream!(
    SolidEntryPoint,
    MilestoneIndex,
    TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX
);
impl_stream!(MilestoneIndex, OutputDiff, TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_stream!(Address, Balance, TREE_ADDRESS_TO_BALANCE);
impl_stream!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE
);
impl_stream!((MilestoneIndex, Receipt), (), TREE_MILESTONE_INDEX_TO_RECEIPT);
impl_stream!((bool, TreasuryOutput), (), TREE_SPENT_TO_TREASURY_OUTPUT);
