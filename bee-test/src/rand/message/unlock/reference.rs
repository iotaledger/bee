// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::number::rand_number_range;

use bee_message::unlock::{ReferenceUnlock, UNLOCK_BLOCK_INDEX_RANGE};

/// Generates a random [`ReferenceUnlock`].
pub fn rand_reference_unlock() -> ReferenceUnlock {
    ReferenceUnlock::new(rand_number_range(UNLOCK_BLOCK_INDEX_RANGE)).unwrap()
}
