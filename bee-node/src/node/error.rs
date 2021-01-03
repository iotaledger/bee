// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid or no identity private key provided. Add the newly generated {0} (or generate one with `bee p2p-identity`) to the configuration file and re-run the node.")]
    InvalidOrNoIdentityPrivateKey(String),
    #[error("Storage backend operation failed: {0}.")]
    StorageBackend(Box<dyn std::error::Error>),
}
