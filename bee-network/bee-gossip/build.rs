// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/proto/handshake.proto",
            "src/proto/packet.proto",
            "src/proto/gossip.proto",
        ],
        &["src/"],
    )?;

    Ok(())
}
