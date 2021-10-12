// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::multiaddr::AutopeeringMultiaddr;

use serde::Deserialize;

use std::net::SocketAddr;

// From Hornet config:
// "autopeering": {
//     "bindAddress": "0.0.0.0:14626",
//     "entryNodes": [
//         "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
//         "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/
// iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",         "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/
// 14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",         "/dns/entry-mainnet.tanglebay.com/udp/14626/
// autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"     ],
//     "entryNodesPreferIPv6": false,
//     "runAsEntryNode": false
// }

#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "autopeering")]
pub struct AutopeeringConfig {
    #[serde(rename = "bindAddress")]
    pub bind_addr: SocketAddr,
    #[serde(rename = "entryNodes")]
    pub entry_nodes: Vec<AutopeeringMultiaddr>,
    #[serde(rename = "entryNodesPreferIPv6")]
    pub entry_nodes_prefer_ipv6: bool,
    #[serde(rename = "runAsEntryNode")]
    pub run_as_entry_node: bool,
}
