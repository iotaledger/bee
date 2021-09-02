// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Truncate access operations.

use crate::{
    column_families::*,
    storage::{Storage, StorageBackend},
};

use bee_message::{Message, MessageId};
use bee_storage::access::Truncate;

fn truncate(storage: &Storage, cf_str: &'static str) -> Result<(), <Storage as StorageBackend>::Error> {
    let cf_handle = storage.cf_handle(cf_str)?;

    let mut iter = storage.inner.raw_iterator_cf(cf_handle);

    // Seek to the first key.
    iter.seek_to_first();
    // Grab the first key if it exists.
    let first = if let Some(first) = iter.key() {
        first.to_vec()
    } else {
        // There are no keys to remove.
        return Ok(());
    };

    iter.seek_to_last();
    // Grab the last key if it exists.
    let last = if let Some(last) = iter.key() {
        let mut last = last.to_vec();
        // `delete_range_cf` excludes the last key in the range so a byte is added to be sure the last key is included.
        last.push(u8::MAX);
        last
    } else {
        // There are no keys to remove.
        return Ok(());
    };

    storage.inner.delete_range_cf(cf_handle, first, last)?;

    Ok(())
}

macro_rules! impl_truncate {
    ($key:ty, $value:ty, $cf:expr) => {
        impl Truncate<$key, $value> for Storage {
            fn truncate(&self) -> Result<(), <Self as StorageBackend>::Error> {
                truncate(self, $cf)
            }
        }
    };
}

impl_truncate!(MessageId, Message, CF_MESSAGE_ID_TO_MESSAGE);
