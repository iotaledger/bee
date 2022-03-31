// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_message::{
    address::Ed25519Address,
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    Message, MessageId,
};
use bee_storage::{access::AsIterator, system::System};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};
use packable::PackableExt;
use rocksdb::{DBIterator, IteratorMode};

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

pub struct StorageIterator<'a, K, V> {
    inner: DBIterator<'a>,
    marker: PhantomData<(K, V)>,
}

impl<'a, K, V> StorageIterator<'a, K, V> {
    fn new(inner: DBIterator<'a>) -> Self {
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
                Ok(StorageIterator::new(
                    self.inner.iterator_cf(self.cf_handle($cf)?, IteratorMode::Start),
                ))
            }
        }

        /// An iterator over all key-value pairs of a column family.
        impl<'a> Iterator for StorageIterator<'a, $key, $value> {
            type Item = Result<($key, $value), <Storage as StorageBackend>::Error>;

            fn next(&mut self) -> Option<Self::Item> {
                self.inner
                    .next()
                    .map(|(key, value)| Ok(Self::unpack_key_value(&key, &value)))

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

impl<'a> StorageIterator<'a, MilestoneIndex, Milestone> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, Milestone) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unverified(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Milestone::unpack_unverified(&mut value).unwrap(),
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

impl_iter!(u8, System, CF_SYSTEM);
impl_iter!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
impl_iter!(MessageId, MessageMetadata, CF_MESSAGE_ID_TO_METADATA);
impl_iter!((MessageId, MessageId), (), CF_MESSAGE_ID_TO_MESSAGE_ID);
impl_iter!(OutputId, CreatedOutput, CF_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_iter!(OutputId, ConsumedOutput, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_iter!(Unspent, (), CF_OUTPUT_ID_UNSPENT);
impl_iter!((Ed25519Address, OutputId), (), CF_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_iter!((), LedgerIndex, CF_LEDGER_INDEX);
impl_iter!(MilestoneIndex, Milestone, CF_MILESTONE_INDEX_TO_MILESTONE);
impl_iter!((), SnapshotInfo, CF_SNAPSHOT_INFO);
impl_iter!(SolidEntryPoint, MilestoneIndex, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX);
impl_iter!(MilestoneIndex, OutputDiff, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_iter!(
    (MilestoneIndex, UnreferencedMessage),
    (),
    CF_MILESTONE_INDEX_TO_UNREFERENCED_MESSAGE
);
impl_iter!((MilestoneIndex, Receipt), (), CF_MILESTONE_INDEX_TO_RECEIPT);
impl_iter!((bool, TreasuryOutput), (), CF_SPENT_TO_TREASURY_OUTPUT);
