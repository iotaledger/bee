// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;
mod password;
#[cfg(feature = "rocksdb")]
mod rocksdb;
#[cfg(feature = "sled")]
mod sled;
mod snapshot_info;

use structopt::StructOpt;
use thiserror::Error;

#[non_exhaustive]
#[derive(Clone, Debug, StructOpt)]
pub enum Tool {
    /// Generates Ed25519 public/private keys and addresses.
    Ed25519(ed25519::Ed25519Tool),
    /// Rocksdb database analyser.
    #[cfg(feature = "rocksdb")]
    Rocksdb(rocksdb::RocksdbTool),
    /// Sled database analyser.
    #[cfg(feature = "sled")]
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
    #[cfg(feature = "rocksdb")]
    #[error("{0}")]
    Rocksdb(#[from] rocksdb::RocksdbError),
    #[cfg(feature = "sled")]
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
        #[cfg(feature = "rocksdb")]
        Tool::Rocksdb(tool) => rocksdb::exec(tool)?,
        #[cfg(feature = "sled")]
        Tool::Sled(tool) => sled::exec(tool)?,
        Tool::SnapshotInfo(tool) => snapshot_info::exec(tool)?,
        Tool::Password(tool) => password::exec(tool)?,
    }

    Ok(())
}
