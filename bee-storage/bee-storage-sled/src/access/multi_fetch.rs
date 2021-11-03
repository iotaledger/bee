// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Multi-fetch access operations.

use crate::{trees::*, Storage};

use bee_message::{Message, MessageId, MessageMetadata};
use bee_packable::Packable;
use bee_storage::{access::MultiFetch, system::System, StorageBackend};

use std::{marker::PhantomData, slice::Iter};

/// Multi-fetch iterator over the database tree.
pub struct DbIter<'a, K, V, E> {
    db: &'a sled::Db,
    keys: Iter<'a, K>,
    marker: PhantomData<(V, E)>,
}

impl<'a, K: Packable, V: Packable, E: From<sled::Error>> Iterator for DbIter<'a, K, V, E> {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        // Packing to bytes can't fail.
        let key = self.keys.next()?.pack_to_vec();

        Some(
            self.db
                .get(key)
                // Unpacking from storage slice can't fail.
                .map(|option| option.map(|bytes| V::unpack(&mut bytes.as_ref()).unwrap()))
                .map_err(E::from),
        )
    }
}

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

/// Multi-fetch iterator over an inner tree.
pub struct TreeIter<'a, K, V, E> {
    tree: sled::Tree,
    keys: Iter<'a, K>,
    marker: PhantomData<(V, E)>,
}

impl<'a, K: Packable, V: Packable, E: From<sled::Error>> Iterator for TreeIter<'a, K, V, E> {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        // Packing to bytes can't fail.
        let key = self.keys.next()?.pack_to_vec();

        Some(
            self.tree
                .get(key)
                // Unpacking from storage slice can't fail.
                .map(|option| option.map(|bytes| V::unpack(&mut bytes.as_ref()).unwrap()))
                .map_err(E::from),
        )
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
impl_multi_fetch!(MessageId, MessageMetadata, TREE_MESSAGE_ID_TO_MESSAGE_METADATA);
