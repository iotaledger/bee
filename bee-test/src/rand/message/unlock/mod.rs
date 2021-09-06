// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;
mod signature;

use crate::rand::number::rand_number;

pub use reference::rand_reference_unlock;
pub use signature::rand_signature_unlock;

use bee_message::unlock::{ReferenceUnlock, SignatureUnlock, UnlockBlock};

/// Generates a random [`UnlockBlock`].
pub fn rand_unlock() -> UnlockBlock {
    match rand_number::<u8>() % 2 {
        SignatureUnlock::KIND => rand_signature_unlock().into(),
        ReferenceUnlock::KIND => rand_reference_unlock().into(),
        _ => unreachable!(),
    }
}
