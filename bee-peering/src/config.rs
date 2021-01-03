// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::manual::{ManualPeeringConfig, ManualPeeringConfigBuilder};

use bee_network::Keypair;

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct PeeringConfigBuilder {
    identity_private_key: Option<String>,
    manual: ManualPeeringConfigBuilder,
}

impl PeeringConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn identity_private_key(mut self, identity_private_key: String) -> Self {
        self.identity_private_key.replace(identity_private_key);
        self
    }

    // TODO better error handling
    pub fn finish(self) -> PeeringConfig {
        let (identity, identity_string, new) = if let Some(identity_string) = self.identity_private_key {
            if identity_string.len() == 128 {
                let mut decoded = [0u8; 64];
                hex::decode_to_slice(&identity_string[..], &mut decoded).expect("error decoding identity");
                let identity = Keypair::decode(&mut decoded).expect("error decoding identity");
                (identity, identity_string, false)
            } else if identity_string.is_empty() {
                generate_random_identity()
            } else {
                panic!("invalid identity string length");
            }
        } else {
            generate_random_identity()
        };

        PeeringConfig {
            identity_private_key: (identity, identity_string, new),
            manual: self.manual.finish(),
        }
    }
}

fn generate_random_identity() -> (Keypair, String, bool) {
    let identity = Keypair::generate();
    let encoded = identity.encode();
    let identity_string = hex::encode(encoded);
    (identity, identity_string, true)
}

#[derive(Clone)]
pub struct PeeringConfig {
    pub identity_private_key: (Keypair, String, bool),
    pub manual: ManualPeeringConfig,
}

impl PeeringConfig {
    pub fn build() -> PeeringConfigBuilder {
        PeeringConfigBuilder::new()
    }
}
