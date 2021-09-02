// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{message::rand_message_id, vec::vec_rand_length};

use bee_message::parents::{ParentsBlock, ParentsKind, PREFIXED_PARENTS_LENGTH_MAX, PREFIXED_PARENTS_LENGTH_MIN};

/// Generates a random [`ParentsBlock`] of a given [`ParentsKind`].
pub fn rand_parents_block(block_type: ParentsKind) -> ParentsBlock {
    let mut parent_ids = vec_rand_length(
        PREFIXED_PARENTS_LENGTH_MIN as usize..=PREFIXED_PARENTS_LENGTH_MAX as usize,
        rand_message_id,
    );

    parent_ids.sort();

    ParentsBlock::new(
        block_type,
        vec_rand_length(
            PREFIXED_PARENTS_LENGTH_MIN as usize..=PREFIXED_PARENTS_LENGTH_MAX as usize,
            rand_message_id,
        ),
    )
    .unwrap()
}
