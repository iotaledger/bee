// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;
mod p2p_identity;
mod rocksdb;
mod snapshot_info;

use structopt::StructOpt;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, StructOpt)]
pub enum Tool {
    /// Generates Ed25519 public/private keys and addresses.
    Ed25519(ed25519::Ed25519Tool),
    /// Generates a p2p identity.
    P2pIdentity(p2p_identity::P2pIdentityTool),
    /// Rocksdb database analyser.
    Rocksdb(rocksdb::RocksdbTool),
    /// Outputs information about a snapshot file.
    SnapshotInfo(snapshot_info::SnapshotInfoTool),
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("{0}")]
    Ed25519(#[from] ed25519::Ed25519Error),
    #[error("{0}")]
    Rocksdb(#[from] rocksdb::RocksdbError),
    #[error("{0}")]
    SnapshotInfo(#[from] snapshot_info::SnapshotInfoError),
}

pub fn exec(tool: &Tool) -> Result<(), ToolError> {
    match tool {
        Tool::Ed25519(tool) => ed25519::exec(tool)?,
        Tool::P2pIdentity(tool) => p2p_identity::exec(tool),
        Tool::Rocksdb(tool) => rocksdb::exec(tool)?,
        Tool::SnapshotInfo(tool) => snapshot_info::exec(tool)?,
    }

    Ok(())
}
