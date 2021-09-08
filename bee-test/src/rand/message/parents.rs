// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    message::rand_message_id,
    number::{rand_number, rand_number_range},
    vec::rand_vec,
};

use bee_message::parents::{ParentsBlock, ParentsKind, PREFIXED_PARENTS_LENGTH_MAX, PREFIXED_PARENTS_LENGTH_MIN};

/// Generates a random [`ParentsBlock`] of a given [`ParentsKind`].
pub fn rand_parents_block(block_type: ParentsKind) -> ParentsBlock {
    let mut parent_ids = rand_vec(
        rand_message_id,
        rand_number_range(PREFIXED_PARENTS_LENGTH_MIN as usize..=PREFIXED_PARENTS_LENGTH_MAX as usize),
    );

    parent_ids.sort();

    ParentsBlock::new(block_type, parent_ids).unwrap()
}

/// Generates a random [`Vec`] of [`ParentsBlock`]s.
pub fn rand_parents_blocks() -> Vec<ParentsBlock> {
    std::iter::once(rand_parents_block(ParentsKind::Strong))
        .chain(
            [ParentsKind::Weak, ParentsKind::Disliked, ParentsKind::Liked]
                .iter()
                .take(rand_number::<usize>() % 3 + 1)
                .map(|pk| rand_parents_block(*pk)),
        )
        .collect()
}
