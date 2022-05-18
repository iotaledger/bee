// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{marker::PhantomData, vec::IntoIter};

use bee_common::packable::Packable;
use bee_ledger::types::{Balance, ConsumedOutput, CreatedOutput, OutputDiff};
use bee_message::{
    address::Address,
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    Message, MessageId,
};
use bee_storage::{access::MultiFetch, system::System};
use bee_tangle::{metadata::MessageMetadata, solid_entry_point::SolidEntryPoint};
use parking_lot::RwLockReadGuard;

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

pub struct MultiIter<'a, V, E> {
    iter: IntoIter<Result<Option<Vec<u8>>, rocksdb::Error>>,
    marker: PhantomData<(V, E)>,
    _guard: Option<RwLockReadGuard<'a, ()>>,
}

impl<'a, V: Packable, E: From<rocksdb::Error>> Iterator for MultiIter<'a, V, E> {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.iter
                .next()?
                .map(|option| option.map(|bytes| V::unpack_unchecked(&mut bytes.as_slice()).unwrap()))
                .map_err(E::from),
        )
    }
}

macro_rules! impl_multi_fetch {
    ($key:ty, $value:ty, $cf:expr) => {
        impl<'a> MultiFetch<'a, $key, $value> for Storage {
            type Iter = MultiIter<'a, $value, <Self as StorageBackend>::Error>;

            fn multi_fetch_op(&'a self, keys: &[$key]) -> Result<Self::Iter, <Self as StorageBackend>::Error> {
                let cf = self.cf_handle($cf)?;

                Ok(MultiIter {
                    iter: self
                        .inner
                        .multi_get_cf(keys.iter().map(|k| (cf, k.pack_new())))
                        .into_iter(),
                    marker: PhantomData,
                    _guard: None,
                })
            }
        }
    };
}

impl_multi_fetch!(u8, System, CF_SYSTEM);
impl_multi_fetch!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
impl_multi_fetch!(OutputId, CreatedOutput, CF_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_multi_fetch!(OutputId, ConsumedOutput, CF_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_multi_fetch!(MilestoneIndex, Milestone, CF_MILESTONE_INDEX_TO_MILESTONE);
impl_multi_fetch!(SolidEntryPoint, MilestoneIndex, CF_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX);
impl_multi_fetch!(MilestoneIndex, OutputDiff, CF_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_multi_fetch!(Address, Balance, CF_ADDRESS_TO_BALANCE);

impl<'a> MultiFetch<'a, MessageId, MessageMetadata> for Storage {
    type Iter = MultiIter<'a, MessageMetadata, <Self as StorageBackend>::Error>;

    fn multi_fetch_op(&'a self, keys: &[MessageId]) -> Result<Self::Iter, <Self as StorageBackend>::Error> {
        let cf = self.cf_handle(CF_MESSAGE_ID_TO_METADATA)?;

        Ok(MultiIter {
            iter: self
                .inner
                .multi_get_cf(keys.iter().map(|k| (cf, k.pack_new())))
                .into_iter(),
            marker: PhantomData,
            _guard: Some(self.locks.message_id_to_metadata.read()),
        })
    }
}
