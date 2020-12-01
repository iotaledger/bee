// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;
mod p2p_identity;
mod snapshot_info;

use structopt::StructOpt;

#[non_exhaustive]
#[derive(Debug, StructOpt)]
pub enum Tool {
    /// Generates Ed25519 public/private keys and addresses.
    Ed25519(ed25519::Ed25519Tool),
    /// Generates a p2p identity.
    P2pIdentity(p2p_identity::P2pIdentityTool),
    /// Outputs information about a snapshot file.
    SnapshotInfo(snapshot_info::SnapshotInfo),
}

pub fn exec(tool: &Tool) {
    match tool {
        Tool::Ed25519(tool) => ed25519::exec(tool),
        Tool::P2pIdentity(tool) => p2p_identity::exec(tool),
        Tool::SnapshotInfo(tool) => snapshot_info::exec(tool),
    }
}
