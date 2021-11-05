// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering configuration.

use crate::multiaddr::AutopeeringMultiaddr;

use serde::{Deserialize, Serialize};

use std::net::SocketAddr;

#[rustfmt::skip]
// # Example
// ```json
// "autopeering": {
//     "bindAddress": "0.0.0.0:14626",
//     "entryNodes": [
//         "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
//         "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
//     ],
//     "entryNodesPreferIPv6": false,
//     "runAsEntryNode": false,
//     "dropNeighborsOnSaltUpdate": false
// }
// ```

/// 
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "autopeering")]
pub struct AutopeeringConfig {
    /// The bind address for the server.
    #[serde(rename = "bindAddress")]
    pub bind_addr: SocketAddr,
    /// The entry nodes for bootstrapping.
    #[serde(rename = "entryNodes")]
    pub entry_nodes: Vec<AutopeeringMultiaddr>,
    /// Whether `Ipv4` or `Ipv6` should be preferred in case a hostname supports both.
    #[serde(rename = "entryNodesPreferIPv6")]
    pub entry_nodes_prefer_ipv6: bool,
    /// Whether the node should run as an entry node.
    #[serde(rename = "runAsEntryNode")]
    pub run_as_entry_node: bool,
    /// Whether all neighbors should be disconnected from when the salts are updated.
    #[serde(rename = "dropNeighborsOnSaltUpdate", default)]
    pub drop_neighbors_on_salt_update: bool,
}
