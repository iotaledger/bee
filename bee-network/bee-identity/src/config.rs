// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with network identities.

use crate::identity::LocalId;

use serde::{Deserialize, Serialize};

/// Network configuration.
#[derive(Clone)]
pub struct NetworkIdentityConfig {
    /// The local identity of the node.
    pub local_id: LocalId,
}

impl NetworkIdentityConfig {
    /// Creates a new network identity config.
    pub fn new(local_id: LocalId) -> Self {
        Self { local_id }
    }
}

/// Serializable (and therefore persistable) network configuration data.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "identity")]
pub struct NetworkIdentityConfigBuilder {
    #[serde(rename = "privateKey")]
    private_key: Option<String>,
}

impl NetworkIdentityConfigBuilder {
    /// Sets the private key for gossip layer authentication.
    pub fn private_key(&mut self, private_key: impl ToString) {
        self.private_key.replace(private_key.to_string());
    }
    /// Finishes the builder.
    pub fn finish(self) -> NetworkIdentityConfig {
        NetworkIdentityConfig {
            local_id: LocalId::from_bs58_encoded_private_key(self.private_key.unwrap()),
        }
    }
}
