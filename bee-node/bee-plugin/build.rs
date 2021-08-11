// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/plugin.proto")?;
    Ok(())
}
