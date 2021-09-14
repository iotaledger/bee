// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Multi-fetch access operations.

use crate::{column_families::*, Storage};

use bee_message::{Message, MessageId};
use bee_packable::Packable;
use bee_storage::{access::MultiFetch, system::System, StorageBackend};

use std::{marker::PhantomData, vec::IntoIter};

/// Multi-fetch iterator over the database column family.
pub struct MultiIter<V, E> {
    iter: IntoIter<Result<Option<Vec<u8>>, rocksdb::Error>>,
    marker: PhantomData<(V, E)>,
}

impl<V: Packable, E: From<rocksdb::Error>> Iterator for MultiIter<V, E> {
    type Item = Result<Option<V>, E>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            self.iter
                .next()?
                // Unpacking from storage slice can't fail.
                .map(|option| option.map(|bytes| V::unpack(&mut bytes.as_slice()).unwrap()))
                .map_err(E::from),
        )
    }
}

macro_rules! impl_multi_fetch {
    ($key:ty, $value:ty, $cf:expr) => {
        impl<'a> MultiFetch<'a, $key, $value> for Storage {
            type Iter = MultiIter<$value, <Self as StorageBackend>::Error>;

            fn multi_fetch(&'a self, keys: &[$key]) -> Result<Self::Iter, <Self as StorageBackend>::Error> {
                let cf = self.cf_handle($cf)?;

                Ok(MultiIter {
                    iter: self
                        .inner
                        // Packing to bytes can't fail.
                        .multi_get_cf(keys.iter().map(|k| (cf, k.pack_to_vec().unwrap())))
                        .into_iter(),
                    marker: PhantomData,
                })
            }
        }
    };
}

impl_multi_fetch!(u8, System, CF_SYSTEM);
impl_multi_fetch!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
