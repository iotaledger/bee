// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Multi-fetch access operations.

use crate::{storage::Storage, trees::*};

use core::fmt::Debug;

use bee_packable::packable::Packable;
use bee_message::{
    Message, MessageId,
};
use bee_storage::{access::MultiFetch, backend::StorageBackend, system::System};

use std::{marker::PhantomData, slice::Iter};

/// Multi-fetch iterator over an inner tree.
pub struct TreeIter<'a, K, V, E> {
    tree: sled::Tree,
    keys: Iter<'a, K>,
    marker: PhantomData<(V, E)>,
}

impl<'a, K: Packable, V: Packable, E: From<sled::Error>> Iterator for TreeIter<'a, K, V, E> where <K as bee_packable::Packable>::PackError: Debug {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let key = Packable::pack_to_vec(self.keys.next()?).unwrap();

        Some(
            self.tree
                .get(key)
                .map(|option| option.map(|bytes| V::unpack_from_slice(&mut bytes.as_ref()).unwrap()))
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

impl<'a, K: Packable, V: Packable, E: From<sled::Error>> Iterator for DbIter<'a, K, V, E> where <K as bee_packable::Packable>::PackError: Debug {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let key = Packable::pack_to_vec(self.keys.next()?).unwrap();

        Some(
            self.db
                .get(key)
                .map(|option| option.map(|bytes| V::unpack_from_slice(&mut bytes.as_ref()).unwrap()))
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