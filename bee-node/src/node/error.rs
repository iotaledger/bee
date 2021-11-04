// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "Invalid or no identity private key provided. Add the newly generated {0} (or generate one with `bee p2p-identity`) to the configuration file and re-run the node."
    )]
    InvalidOrNoIdentityPrivateKey(String),

    #[error("Storage backend operation failed: {0}.")]
    StorageBackend(Box<dyn std::error::Error>),

    #[error("Network initialization failed. Cause: {0}")]
    NetworkInitializationFailed(#[from] bee_network::Error),

    #[error("Peering initialization failed. Cause: {0}")]
    PeeringInitializationFailed(Box<dyn std::error::Error>),
}
