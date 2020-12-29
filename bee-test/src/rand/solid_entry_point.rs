// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::message::rand_message_id;

use bee_protocol::tangle::SolidEntryPoint;

pub fn rand_solid_entry_point() -> SolidEntryPoint {
    rand_message_id().into()
}
