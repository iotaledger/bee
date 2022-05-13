// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::solid_entry_point::SolidEntryPoint;

use crate::rand::block::rand_block_id;

/// Generates a random solid entry point.
pub fn rand_solid_entry_point() -> SolidEntryPoint {
    rand_block_id().into()
}
