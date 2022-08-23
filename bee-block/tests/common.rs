// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    constant::{PROTOCOL_VERSION, TOKEN_SUPPLY},
    output::RentStructureBuilder,
    protocol::ProtocolParameters,
};

pub fn protocol_parameters() -> ProtocolParameters {
    ProtocolParameters::new(
        PROTOCOL_VERSION,
        String::from("testnet"),
        String::from("rms"),
        1000,
        15,
        RentStructureBuilder::new()
            .byte_cost(500)
            .key_factor(10)
            .data_factor(1)
            .finish(),
        TOKEN_SUPPLY,
    )
    .unwrap()
}
