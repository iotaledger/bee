// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;
mod p2p_identity;
mod password;
mod rocksdb;
mod sled;
mod snapshot_info;

use structopt::StructOpt;
use thiserror::Error;

#[non_exhaustive]
#[derive(Clone, Debug, StructOpt)]
pub enum Tool {
    /// Generates Ed25519 public/private keys and addresses.
    Ed25519(ed25519::Ed25519Tool),
    /// Generates a p2p identity.
    P2pIdentity(p2p_identity::P2pIdentityTool),
    /// Rocksdb database analyser.
    Rocksdb(rocksdb::RocksdbTool),
    /// Sled database analyser.
    Sled(sled::SledTool),
    /// Outputs information about a snapshot file.
    SnapshotInfo(snapshot_info::SnapshotInfoTool),
    /// Generates password salt and hash.
    Password(password::PasswordTool),
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("{0}")]
    Ed25519(#[from] ed25519::Ed25519Error),
    #[error("{0}")]
    Rocksdb(#[from] rocksdb::RocksdbError),
    #[error("{0}")]
    Sled(#[from] sled::SledError),
    #[error("{0}")]
    SnapshotInfo(#[from] snapshot_info::SnapshotInfoError),
    #[error("{0}")]
    Password(#[from] password::PasswordError),
}

pub fn exec(tool: &Tool) -> Result<(), ToolError> {
    match tool {
        Tool::Ed25519(tool) => ed25519::exec(tool)?,
        Tool::P2pIdentity(tool) => p2p_identity::exec(tool),
        Tool::Rocksdb(tool) => rocksdb::exec(tool)?,
        Tool::Sled(tool) => sled::exec(tool)?,
        Tool::SnapshotInfo(tool) => snapshot_info::exec(tool)?,
        Tool::Password(tool) => password::exec(tool)?,
    }

    Ok(())
}
