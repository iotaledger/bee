// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Multi-fetch access operations.

use crate::{storage::Storage, trees::*};

use bee_common::packable::Packable;
use bee_ledger::types::{Balance, ConsumedOutput, CreatedOutput, OutputDiff};
use bee_message::{
    address::Address,
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    Message, MessageId,
};
use bee_storage::{access::MultiFetch, backend::StorageBackend, system::System};
use bee_tangle::{metadata::MessageMetadata, solid_entry_point::SolidEntryPoint};

use std::{marker::PhantomData, slice::Iter};

/// Multi-fetch iterator over an inner tree.
pub struct TreeIter<'a, K, V, E> {
    tree: sled::Tree,
    keys: Iter<'a, K>,
    marker: PhantomData<(V, E)>,
}

impl<'a, K: Packable, V: Packable, E: From<sled::Error>> Iterator for TreeIter<'a, K, V, E> {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys.next()?.pack_new();

        Some(
            self.tree
                .get(key)
                .map(|option| option.map(|bytes| V::unpack_unchecked(&mut bytes.as_ref()).unwrap()))
                .map_err(E::from),
        )
    }
}

/// Multi-fetch iterator over the database tree.
pub struct DbIter<'a, K, V, E> {
    db: &'a sled::Db,
    keys: Iter<'a, K>,
    marker: PhantomData<(V, E)>,
}

impl<'a, K: Packable, V: Packable, E: From<sled::Error>> Iterator for DbIter<'a, K, V, E> {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let key = self.keys.next()?.pack_new();

        Some(
            self.db
                .get(key)
                .map(|option| option.map(|bytes| V::unpack_unchecked(&mut bytes.as_ref()).unwrap()))
                .map_err(E::from),
        )
    }
}

#[allow(clippy::type_complexity)]
impl<'a> MultiFetch<'a, u8, System> for Storage {
    type Iter = DbIter<'a, u8, System, <Self as StorageBackend>::Error>;

    fn multi_fetch(&'a self, keys: &'a [u8]) -> Result<Self::Iter, <Self as StorageBackend>::Error> {
        Ok(DbIter {
            db: &self.inner,
            keys: keys.iter(),
            marker: PhantomData,
        })
    }
}

macro_rules! impl_multi_fetch {
    ($key:ty, $value:ty, $cf:expr) => {
        impl<'a> MultiFetch<'a, $key, $value> for Storage {
            type Iter = TreeIter<'a, $key, $value, <Self as StorageBackend>::Error>;

            fn multi_fetch(&'a self, keys: &'a [$key]) -> Result<Self::Iter, <Self as StorageBackend>::Error> {
                Ok(TreeIter {
                    tree: self.inner.open_tree($cf)?,
                    keys: keys.iter(),
                    marker: PhantomData,
                })
            }
        }
    };
}

impl_multi_fetch!(MessageId, Message, TREE_MESSAGE_ID_TO_MESSAGE);
impl_multi_fetch!(MessageId, MessageMetadata, TREE_MESSAGE_ID_TO_METADATA);
impl_multi_fetch!(OutputId, CreatedOutput, TREE_OUTPUT_ID_TO_CREATED_OUTPUT);
impl_multi_fetch!(OutputId, ConsumedOutput, TREE_OUTPUT_ID_TO_CONSUMED_OUTPUT);
impl_multi_fetch!(MilestoneIndex, Milestone, TREE_MILESTONE_INDEX_TO_MILESTONE);
impl_multi_fetch!(
    SolidEntryPoint,
    MilestoneIndex,
    TREE_SOLID_ENTRY_POINT_TO_MILESTONE_INDEX
);
impl_multi_fetch!(MilestoneIndex, OutputDiff, TREE_MILESTONE_INDEX_TO_OUTPUT_DIFF);
impl_multi_fetch!(Address, Balance, TREE_ADDRESS_TO_BALANCE);
