// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides features related to ordering.

#![no_std]
#![warn(missing_docs)]

use core::{cmp::Ordering, iter::Iterator};

/// Checks if an iterator yields ordered and unique values.
pub fn is_unique_sorted<T: Ord, I: Iterator<Item = T>>(mut iterator: I) -> bool {
    let mut previous = match iterator.next() {
        Some(e) => e,
        None => return true,
    };

    for curr in iterator {
        if previous.cmp(&curr) != Ordering::Less {
            return false;
        }
        previous = curr;
    }

    true
}

/// Checks if an iterator yields ordered values.
pub fn is_sorted<T: Ord, I: Iterator<Item = T>>(mut iterator: I) -> bool {
    let mut previous = match iterator.next() {
        Some(e) => e,
        None => return true,
    };

    for curr in iterator {
        if previous.cmp(&curr) == Ordering::Greater {
            return false;
        }
        previous = curr;
    }

    true
}
