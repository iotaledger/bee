// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/proto/discovery.proto",
            "src/proto/peering.proto",
            "src/proto/packet.proto",
            "src/proto/peer.proto",
            "src/proto/salt.proto",
            "src/proto/service.proto",
        ],
        &["src/"],
    )?;

    Ok(())
}
