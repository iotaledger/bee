// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::rand::{block::rand_block_id, milestone::rand_milestone_id, number::rand_number};

use crate::milestone_metadata::MilestoneMetadata;

/// Generates a random milestone metadata.
pub fn rand_milestone_metadata() -> MilestoneMetadata {
    MilestoneMetadata::new(rand_block_id(), rand_milestone_id(), rand_number())
}
