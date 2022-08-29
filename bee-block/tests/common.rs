// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{output::RentStructure, protocol::ProtocolParameters};

pub fn protocol_parameters() -> ProtocolParameters {
    ProtocolParameters::new(
        2,
        String::from("testnet"),
        String::from("rms"),
        1500,
        15,
        RentStructure::build()
            .byte_cost(500)
            .key_factor(10)
            .data_factor(1)
            .finish(),
        1_813_620_509,
    )
    .unwrap()
}
