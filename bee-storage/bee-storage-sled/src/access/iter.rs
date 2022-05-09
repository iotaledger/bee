// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Iter access operations.

use std::marker::PhantomData;

use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_message::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Message, MessageId,
};
use bee_storage::{access::AsIterator, backend::StorageBackend, system::System};
use bee_tangle::{
    message_metadata::MessageMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_message::UnreferencedMessage,
};
use packable::PackableExt;

use crate::{storage::Storage, trees::*};

/// Type used to iterate a subtree.
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

macro_rules! impl_iter {
    ($key:ty, $value:ty, $cf:expr) => {
        impl<'a> AsIterator<'a, $key, $value> for Storage {
            type AsIter = StorageIterator<'a, $key, $value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                Ok(StorageIterator::new(self.inner.open_tree($cf)?.iter()))
            }
        }

        /// An iterator over all key-value pairs of a column family.
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
            u8::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            System::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MessageId, Message> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, Message) {
        (
            // Unpacking from storage is fine.
            MessageId::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Message::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MessageId, MessageMetadata> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, MessageMetadata) {
        (
            // Unpacking from storage is fine.
            MessageId::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MessageMetadata::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (MessageId, MessageId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MessageId, MessageId), ()) {
        let (mut parent, mut child) = key.split_at(MessageId::LENGTH);

        (
            (
                // Unpacking from storage is fine.
                MessageId::unpack_unverified(&mut parent).unwrap(),
                // Unpacking from storage is fine.
                MessageId::unpack_unverified(&mut child).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageIterator<'a, OutputId, CreatedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, CreatedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            CreatedOutput::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, OutputId, ConsumedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, ConsumedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            ConsumedOutput::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, Unspent, ()> {
    fn unpack_key_value(mut key: &[u8], _: &[u8]) -> (Unspent, ()) {
        (
            // Unpacking from storage is fine.
            Unspent::unpack_unverified(&mut key).unwrap(),
            (),
        )
    }
}

impl<'a> StorageIterator<'a, (Ed25519Address, OutputId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((Ed25519Address, OutputId), ()) {
        let (mut address, mut output_id) = key.split_at(MessageId::LENGTH);

        (
            (
                // Unpacking from storage is fine.
                Ed25519Address::unpack_unverified(&mut address).unwrap(),
                // Unpacking from storage is fine.
                OutputId::unpack_unverified(&mut output_id).unwrap(),
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
            LedgerIndex::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneIndex, MilestoneMetadata> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, MilestoneMetadata) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MilestoneMetadata::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneId, MilestonePayload> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneId, MilestonePayload) {
        (
            // Unpacking from storage is fine.
            MilestoneId::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MilestonePayload::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (), SnapshotInfo> {
    fn unpack_key_value(_: &[u8], mut value: &[u8]) -> ((), SnapshotInfo) {
        (
            (),
            // Unpacking from storage is fine.
            SnapshotInfo::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, SolidEntryPoint, MilestoneIndex> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (SolidEntryPoint, MilestoneIndex) {
        (
            // Unpacking from storage is fine.
            SolidEntryPoint::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneIndex, OutputDiff> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, OutputDiff) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            OutputDiff::unpack_unverified(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (MilestoneIndex, UnreferencedMessage), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MilestoneIndex, UnreferencedMessage), ()) {
        let (mut index, mut unreferenced_message) = key.split_at(std::mem::size_of::<MilestoneIndex>());

        (
            (
                // Unpacking from storage is fine.
                MilestoneIndex::unpack_unverified(&mut index).unwrap(),
                // Unpacking from storage is fine.
                UnreferencedMessage::unpack_unverified(&mut unreferenced_message).unwrap(),
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
                MilestoneIndex::unpack_unverified(&mut index).unwrap(),
                // Unpacking from storage is fine.
                Receipt::unpack_unverified(&mut receipt).unwrap(),
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
                bool::unpack_unverified(&mut index).unwrap(),
                // Unpacking from storage is fine.
                TreasuryOutput::unpack_unverified(&mut receipt).unwrap(),
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

/// An iterator over all key-value pairs of a column family.
impl<'a> Iterator for StorageIterator<'a, u8, System> {
    type Item = Result<(u8, System), <Storage as StorageBackend>::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result
                .map(|(key, value)| Self::unpack_key_value(&key, &value))
                .map_err(From::from)
        })
    }
}

impl_iter!(MessageId, Message, TREE_MESSAGE_ID_TO_MESSAGE);
impl_iter!(MessageId, MessageMetadata, TREE_MESSAGE_ID_TO_METADATA);
impl_iter!((MessageId, MessageId), (), TREE_MESSAGE_ID_TO_MESSAGE_ID);
impl_iter!(OutputId, CreatedOutput, TREE_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_iter!(OutputId, ConsumedOutput, TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_iter!(Unspent, (), TREE_OUTPUT_ID_UNSPENT);
impl_iter!((Ed25519Address, OutputId), (), TREE_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_iter!((), LedgerIndex, TREE_LEDGER_INDEX);
impl_iter!(
    MilestoneIndex,
    MilestoneMetadata,
    TREE_MILESTONE_INDEX_TO_MILESTONE_METADATA
);
impl_iter!(MilestoneId, MilestonePayload, TREE_MILESTONE_ID_TO_MILESTONE_PAYLOAD);
impl_iter!((), SnapshotInfo, TREE_SNAPSHOT_INFO);
impl_iter!(
    SolidEntryPoint,
    MilestoneIndex,
    TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX
);
impl_iter!(MilestoneIndex, OutputDiff, TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_iter!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    TREE_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE
);
impl_iter!((MilestoneIndex, Receipt), (), TREE_MILESTONE_INDEX_TO_RECEIPT);
impl_iter!((bool, TreasuryOutput), (), TREE_SPENT_TO_TREASURY_OUTPUT);
