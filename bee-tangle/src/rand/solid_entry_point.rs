// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::rand::block::rand_block_id;

use crate::solid_entry_point::SolidEntryPoint;

/// Generates a random solid entry point.
pub fn rand_solid_entry_point() -> SolidEntryPoint {
    rand_block_id().into()
}
