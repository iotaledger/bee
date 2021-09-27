// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering configuration.

use serde::{Deserialize, Serialize};

/// Manual peer configuration.
#[derive(Clone)]
pub struct AutoPeeringConfig {
    // TODO
}

impl AutoPeeringConfig {
    // TODO
}

/// Serializable representation of a list of manual peers.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename = "manualPeering")]
pub struct AutoPeeringConfigBuilder {
    #[serde(rename = "bindAddress")]
    bind_addr: Option<()>,
    #[serde(rename = "entryNodes")]
    entry_nodes: Option<Vec<()>>,
}

impl AutoPeeringConfigBuilder {
    /// Finishes the builder.
    pub fn finish(self) -> AutoPeeringConfig {
        AutoPeeringConfig {}
    }
}
