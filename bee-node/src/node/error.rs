// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid or no keypair provided. Add the newly generated keypair {0} (or generate one with `bee p2p-identity`) to the configuration file and re-run the node.")]
    InvalidOrNoKeypair(String),
    #[error("Storage backend operation failed: {0}.")]
    StorageBackend(Box<dyn std::error::Error>),
}
