// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::rand::block::rand_block_id;

use crate::unreferenced_block::UnreferencedBlock;

/// Generates a random unreferenced block.
pub fn rand_unreferenced_block() -> UnreferencedBlock {
    rand_block_id().into()
}
