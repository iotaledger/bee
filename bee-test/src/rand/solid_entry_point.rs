// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::message::rand_message_id;

use bee_tangle::solid_entry_point::SolidEntryPoint;

/// Generates a random solid entry point.
pub fn rand_solid_entry_point() -> SolidEntryPoint {
    rand_message_id().into()
}
