// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Stream access operations.

use crate::{storage::Storage, trees::*};

use bee_packable::packable::Packable;

use bee_message::{Message, MessageId};
use bee_storage::{access::AsIterator, backend::StorageBackend, system::System};

use std::marker::PhantomData;

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
            u8::unpack_from_slice(&mut key).unwrap(),
            // Unpacking from storage is fine.
            System::unpack_from_slice(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MessageId, Message> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, Message) {
        (
            // Unpacking from storage is fine.
            MessageId::unpack_from_slice(&mut key).unwrap(),
            // Unpacking from storage is fine.
            Message::unpack_from_slice(&mut value).unwrap(),
        )
    }
}

impl_stream!(MessageId, Message, TREE_MESSAGE_ID_TO_MESSAGE);
