// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

/// A type representing errors that may occur when creating a node.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid or no identity private key provided.
    #[error(
        "Invalid or no identity private key provided. Add the newly generated {0} (or generate one with `bee p2p-identity`) to the configuration file and re-run the node."
    )]
    InvalidOrNoIdentityPrivateKey(String),
    /// Storage backend operation failed.
    #[error("Storage backend operation failed: {0}.")]
    StorageBackend(Box<dyn std::error::Error>),
}
