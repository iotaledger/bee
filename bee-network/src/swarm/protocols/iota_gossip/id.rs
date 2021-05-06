// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p_core::ProtocolName;

use std::fmt;

#[derive(Debug, Clone)]
pub struct IotaGossipIdentifier {
    name: String,
    network_id: u64,
    version: String,
    buffered: Vec<u8>,
}

impl IotaGossipIdentifier {
    pub fn new(name: String, network_id: u64, version: String) -> Self {
        let buffered = format!("/{}/{}/{}", name, network_id, version).into_bytes();

        Self {
            name,
            network_id,
            version,
            buffered,
        }
    }
}

impl ProtocolName for IotaGossipIdentifier {
    fn protocol_name(&self) -> &[u8] {
        &self.buffered
    }
}

impl fmt::Display for IotaGossipIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Panic:
        // Unwrapping is fine, because we made sure `buffered` contains a valid Utf8 string.
        write!(f, "{}", String::from_utf8(self.buffered.clone()).unwrap())
    }
}
