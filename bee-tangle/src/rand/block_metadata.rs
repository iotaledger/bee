// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    rand::{block::rand_block_id, milestone::rand_milestone_index, number::rand_number, option::rand_option},
    semantic::ConflictReason,
};

use crate::{
    block_metadata::{BlockMetadata, IndexId},
    flags::Flags,
};

/// Generates a random conflict reason.
/// It leaves out [`ConflictReason::SemanticValidationFailed`] as it is just a placeholder defined by the protocol but
/// is not actually being used within the bee framework.
pub fn rand_conflict_reason() -> ConflictReason {
    ((rand_number::<u64>() % 6) as u8).try_into().unwrap()
}

/// Generates a random block metadata.
pub fn rand_block_metadata() -> BlockMetadata {
    BlockMetadata::new(
        unsafe { Flags::from_bits_unchecked(rand_number::<u8>()) },
        rand_option(rand_milestone_index()),
        rand_number(),
        rand_number(),
        rand_number(),
        rand_option((
            IndexId::new(rand_milestone_index(), rand_block_id()),
            IndexId::new(rand_milestone_index(), rand_block_id()),
        )),
        rand_conflict_reason(),
        rand_option(rand_number()),
    )
}
