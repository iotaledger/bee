// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::{cmp::Ordering, iter::Iterator};

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
