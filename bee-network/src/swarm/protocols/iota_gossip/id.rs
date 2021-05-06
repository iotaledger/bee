// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p_core::ProtocolName;

use std::fmt;

#[derive(Debug, Clone)]
pub struct IotaGossipIdentifier(String);

impl IotaGossipIdentifier {
    pub fn new(name: impl AsRef<str>, network_id: u64, version: impl AsRef<str>) -> Self {
        Self(format!("/{}/{}/{}", name.as_ref(), network_id, version.as_ref()))
    }
}

impl ProtocolName for IotaGossipIdentifier {
    fn protocol_name(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for IotaGossipIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
