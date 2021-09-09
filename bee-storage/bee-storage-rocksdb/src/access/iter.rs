// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Iterator access operations.

use crate::{column_families::*, Storage};

use bee_message::{Message, MessageId};
use bee_packable::Packable;
use bee_storage::{access::AsIterator, system::System, StorageBackend};

use rocksdb::{DBIterator, IteratorMode};

use std::marker::PhantomData;

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

macro_rules! impl_stream {
    ($key:ty, $value:ty, $cf:expr) => {
        impl<'a> AsIterator<'a, $key, $value> for Storage {
            type AsIter = StorageIterator<'a, $key, $value>;

            fn iter(&'a self) -> Result<Self::AsIter, <Self as StorageBackend>::Error> {
                Ok(StorageIterator::new(
                    self.inner.iterator_cf(self.cf_handle($cf)?, IteratorMode::Start),
                ))
            }
        }

        /// A stream to iterate over all key-value pairs of a column family.
        impl<'a> Iterator for StorageIterator<'a, $key, $value> {
            type Item = Result<($key, $value), <Storage as StorageBackend>::Error>;

            fn next(&mut self) -> Option<Self::Item> {
                let item = self
                    .inner
                    .next()
                    .map(|(key, value)| Ok(Self::unpack_key_value(&key, &value)));

                if let Err(e) = self.inner.status() {
                    Some(Err(e.into()))
                } else {
                    item
                }
            }
        }
    };
}

impl<'a> StorageIterator<'a, u8, System> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (u8, System) {
        (
            // Unpacking from storage slice can't fail.
            u8::unpack(&mut key).unwrap(),
            // Unpacking from storage slice can't fail.
            System::unpack(&mut value).unwrap(),
        )
    }
}

impl<'a> StorageIterator<'a, MessageId, Message> {
    fn unpack_key_value(mut key: &[u8], mut value: &[u8]) -> (MessageId, Message) {
        (
            // Unpacking from storage slice can't fail.
            MessageId::unpack(&mut key).unwrap(),
            // Unpacking from storage slice can't fail.
            Message::unpack(&mut value).unwrap(),
        )
    }
}

impl_stream!(u8, System, CF_SYSTEM);
impl_stream!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
