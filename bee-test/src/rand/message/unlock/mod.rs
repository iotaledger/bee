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

/// Generates a list of random [`UnlockBlock`]s.
pub fn rand_unlocks(len: usize) -> Vec<UnlockBlock> {
    let mut unlock_blocks = Vec::with_capacity(len);
    let mut signature_blocks = vec![];

    for _ in 0..len {
        let unlock_block = match rand_number::<u8>() % 2 {
            SignatureUnlock::KIND => rand_signature_unlock().into(),
            ReferenceUnlock::KIND => {
                if signature_blocks.is_empty() {
                    signature_blocks.push(unlock_blocks.len());
                    rand_signature_unlock().into()
                } else {
                    let signature_block_idx = rand_number::<usize>() % signature_blocks.len();
                    let signature_block = signature_blocks[signature_block_idx];

                    ReferenceUnlock::new(signature_block as u16).unwrap().into()
                }
            }
            _ => unreachable!(),
        };

        unlock_blocks.push(unlock_block);
    }

    unlock_blocks
}
