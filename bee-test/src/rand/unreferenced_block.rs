// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::unreferenced_block::UnreferencedBlock;

use crate::rand::block::rand_block_id;

/// Generates a random unrefere,ced block.
pub fn rand_unreferenced_block() -> UnreferencedBlock {
    rand_block_id().into()
}
