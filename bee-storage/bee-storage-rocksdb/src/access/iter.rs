// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use bee_block::{
    address::Ed25519Address,
    output::OutputId,
    payload::milestone::{MilestoneId, MilestoneIndex, MilestonePayload},
    Block, BlockId,
};
use bee_ledger::types::{
    snapshot::SnapshotInfo, ConsumedOutput, CreatedOutput, LedgerIndex, OutputDiff, Receipt, TreasuryOutput, Unspent,
};
use bee_storage::{access::AsIterator, system::System};
use bee_tangle::{
    block_metadata::BlockMetadata, milestone_metadata::MilestoneMetadata, solid_entry_point::SolidEntryPoint,
    unreferenced_block::UnreferencedBlock,
};
use packable::PackableExt;
use parking_lot::RwLockReadGuard;
use rocksdb::{DBIterator, IteratorMode};

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

pub struct StorageIterator<'a, K, V> {
    inner: DBIterator<'a>,
    marker: PhantomData<(K, V)>,
    _guard: Option<RwLockReadGuard<'a, ()>>,
}

impl<'a, K, V> StorageIterator<'a, K, V> {
    fn new(inner: DBIterator<'a>, guard: Option<RwLockReadGuard<'a, ()>>) -> Self {
        StorageIterator::<K, V> {
            inner,
            marker: PhantomData,
            _guard: guard,
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
                    None,
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
            u8::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            System::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, BlockId, Block> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (BlockId, Block) {
        (
            // Unpacking from storage is fine.
            BlockId::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            Block::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, BlockId, BlockMetadata> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (BlockId, BlockMetadata) {
        (
            // Unpacking from storage is fine.
            BlockId::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            BlockMetadata::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (BlockId, BlockId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((BlockId, BlockId), ()) {
        let (mut parent, mut child) = key.split_at(BlockId::LENGTH);

        (
            (
                // Unpacking from storage is fine.
                BlockId::unpack_unverified(&mut parent, &mut ()).unwrap(),
                // Unpacking from storage is fine.
                BlockId::unpack_unverified(&mut child, &mut ()).unwrap(),
            ),
            (),
        )
    }
}

impl<'a> StorageIterator<'a, OutputId, CreatedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, CreatedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            CreatedOutput::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, OutputId, ConsumedOutput> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (OutputId, ConsumedOutput) {
        (
            // Unpacking from storage is fine.
            OutputId::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            ConsumedOutput::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, Unspent, ()> {
    fn unpack_key_value(mut key: &[u8], _: &[u8]) -> (Unspent, ()) {
        (
            // Unpacking from storage is fine.
            Unspent::unpack_unverified(&mut key, &mut ()).unwrap(),
            (),
        )
    }
}

impl<'a> StorageIterator<'a, (Ed25519Address, OutputId), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((Ed25519Address, OutputId), ()) {
        let (mut address, mut output_id) = key.split_at(BlockId::LENGTH);

        (
            (
                // Unpacking from storage is fine.
                Ed25519Address::unpack_unverified(&mut address, &mut ()).unwrap(),
                // Unpacking from storage is fine.
                OutputId::unpack_unverified(&mut output_id, &mut ()).unwrap(),
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
            LedgerIndex::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneIndex, MilestoneMetadata> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, MilestoneMetadata) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            MilestoneMetadata::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneId, MilestonePayload> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneId, MilestonePayload) {
        (
            // Unpacking from storage is fine.
            MilestoneId::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            MilestonePayload::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (), SnapshotInfo> {
    fn unpack_key_value(_: &[u8], mut value: &[u8]) -> ((), SnapshotInfo) {
        (
            (),
            // Unpacking from storage is fine.
            SnapshotInfo::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, SolidEntryPoint, MilestoneIndex> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (SolidEntryPoint, MilestoneIndex) {
        (
            // Unpacking from storage is fine.
            SolidEntryPoint::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MilestoneIndex, OutputDiff> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MilestoneIndex, OutputDiff) {
        (
            // Unpacking from storage is fine.
            MilestoneIndex::unpack_unverified(&mut key, &mut ()).unwrap(),
            // Unpacking from storage is fine.
            OutputDiff::unpack_unverified(&mut value, &mut ()).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, (MilestoneIndex, UnreferencedBlock), ()> {
    fn unpack_key_value(key: &[u8], _: &[u8]) -> ((MilestoneIndex, UnreferencedBlock), ()) {
        let (mut index, mut unreferenced_block) = key.split_at(std::mem::size_of::<MilestoneIndex>());

        (
            (
                // Unpacking from storage is fine.
                MilestoneIndex::unpack_unverified(&mut index, &mut ()).unwrap(),
                // Unpacking from storage is fine.
                UnreferencedBlock::unpack_unverified(&mut unreferenced_block, &mut ()).unwrap(),
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
                MilestoneIndex::unpack_unverified(&mut index, &mut ()).unwrap(),
                // Unpacking from storage is fine.
                Receipt::unpack_unverified(&mut receipt, &mut ()).unwrap(),
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
                bool::unpack_unverified(&mut index, &mut ()).unwrap(),
                // Unpacking from storage is fine.
                TreasuryOutput::unpack_unverified(&mut receipt, &mut ()).unwrap(),
            ),
            (),
        )
    }
}

impl_iter!(u8, System, CF_SYSTEM);
impl_iter!(BlockId, Block, CF_BLOCK_ID_TO_BLOCK);
impl_iter!((BlockId, BlockId), (), CF_BLOCK_ID_TO_BLOCK_ID);
impl_iter!(OutputId, CreatedOutput, CF_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_iter!(OutputId, ConsumedOutput, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_iter!(Unspent, (), CF_OUTPUT_ID_UNSPENT);
impl_iter!((Ed25519Address, OutputId), (), CF_ED25519_ADDRESS_TO_OUTPUT_ID);
impl_iter!((), LedgerIndex, CF_LEDGER_INDEX);
impl_iter!(
    MilestoneIndex,
    MilestoneMetadata,
    CF_MILESTONE_INDEX_TO_MILESTONE_METADATA
);
impl_iter!(MilestoneId, MilestonePayload, CF_MILESTONE_ID_TO_MILESTONE_PAYLOAD);
impl_iter!((), SnapshotInfo, CF_SNAPSHOT_INFO);
impl_iter!(SolidEntryPoint, MilestoneIndex, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX);
impl_iter!(MilestoneIndex, OutputDiff, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_iter!(
    (MilestoneIndex, UnreferencedBlock),
    (),
    CF_MILESTONE_INDEX_TO_UNREFERENCED_BLOCK
);
impl_iter!((MilestoneIndex, Receipt), (), CF_MILESTONE_INDEX_TO_RECEIPT);
impl_iter!((bool, TreasuryOutput), (), CF_SPENT_TO_TREASURY_OUTPUT);

impl<'a> AsIterator<'a, BlockId, BlockMetadata> for Storage {
    type AsIter = StorageIterator<'a, BlockId, BlockMetadata>;

    fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
        Ok(StorageIterator::new(
            self.inner
                .iterator_cf(self.cf_handle(CF_BLOCK_ID_TO_METADATA)?, IteratorMode::Start),
            Some(self.locks.block_id_to_metadata.read()),
        ))
    }
}

/// An iterator over all key-value pairs of a column family.
impl<'a> Iterator for StorageIterator<'a, BlockId, BlockMetadata> {
    type Item = Result<(BlockId, BlockMetadata), <Storage as StorageBackend>::Error>;

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
