// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{output::RentStructure, protocol::ProtocolParameters};

pub fn protocol_parameters() -> ProtocolParameters {
    ProtocolParameters::new(
        0,
        String::from(""),
        String::from(""),
        0,
        0,
        RentStructure::build()
            .byte_cost(0)
            .key_factor(0)
            .data_factor(0)
            .finish(),
        0,
    )
    .unwrap()
}
