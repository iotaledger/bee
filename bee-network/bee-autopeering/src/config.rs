// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering configuration.
//!
//! ## JSON Example
//! ```json
//! "autopeering": {
//!     "enabled": true,
//!     "bindAddress": "0.0.0.0:14626",
//!     "entryNodes": [
//!          "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
//!          "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
//!          "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
//!          "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
//!     ],
//!     "entryNodesPreferIPv6": true,
//! }
//! ```
//!
//! ## TOML Example
//!
//! ```toml
//! [autopeering]
//! enabled = true
//! bind_address = "0.0.0.0:15626"
//! entry_nodes = [
//!     "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
//!     "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
//!     "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
//!     "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
//! ]
//! entry_nodes_prefer_ipv6 = true
//! ```

use crate::multiaddr::AutopeeringMultiaddr;

use serde::{Deserialize, Serialize};

use std::{
    fmt::Debug,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

// TODO: watch out for possible constification regarding `SocketAddr::new()`.
const AUTOPEERING_BIND_ADDR_DEFAULT: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
const AUTOPEERING_BIND_PORT_DEFAULT: u16 = 0;
const ENTRYNODES_PREFER_IPV6_DEFAULT: bool = false;
const RUN_AS_ENTRYNODE_DEFAULT: bool = false;
const DROP_NEIGHBORS_ON_SALT_UPDATE_DEFAULT: bool = false;

/// The autopeering config.
#[derive(Clone, Debug)]
pub struct AutopeeringConfig {
    enabled: bool,
    bind_addr: SocketAddr,
    entry_nodes: Vec<AutopeeringMultiaddr>,
    entry_nodes_prefer_ipv6: bool,
    run_as_entry_node: bool,
    drop_neighbors_on_salt_update: bool,
}

impl AutopeeringConfig {
    /// Wether autopeering should be enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// The bind address for the server.
    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    /// The entry nodes for bootstrapping.
    pub fn entry_nodes(&self) -> &[AutopeeringMultiaddr] {
        &self.entry_nodes
    }

    /// Whether `Ipv4` or `Ipv6` should be preferred in case a hostname supports both.
    pub fn entry_nodes_prefer_ipv6(&self) -> bool {
        self.entry_nodes_prefer_ipv6
    }

    /// Whether the node should run as an entry node.
    pub fn run_as_entry_node(&self) -> bool {
        self.run_as_entry_node
    }

    /// Whether all neighbors should be disconnected from when the salts are updated.
    pub fn drop_neighbors_on_salt_update(&self) -> bool {
        self.drop_neighbors_on_salt_update
    }

    /// Reduces this config to its list of entry node addresses.
    pub fn into_entry_nodes(self) -> Vec<AutopeeringMultiaddr> {
        self.entry_nodes
    }

    /// Turns the [`AutopeeringConfig`] into its JSON representation.
    pub fn into_json_config(self) -> AutopeeringConfigJsonBuilder {
        AutopeeringConfigJsonBuilder {
            enabled: self.enabled,
            bind_addr: self.bind_addr,
            entry_nodes: self.entry_nodes,
            entry_nodes_prefer_ipv6: Some(self.entry_nodes_prefer_ipv6),
            run_as_entry_node: Some(self.run_as_entry_node),
            drop_neighbors_on_salt_update: Some(self.drop_neighbors_on_salt_update),
        }
    }

    /// Turns the [`AutopeeringConfig`] into its TOML representation.
    pub fn into_toml_config(self) -> AutopeeringConfigTomlBuilder {
        AutopeeringConfigTomlBuilder {
            enabled: self.enabled,
            bind_addr: self.bind_addr,
            entry_nodes: self.entry_nodes,
            entry_nodes_prefer_ipv6: Some(self.entry_nodes_prefer_ipv6),
            run_as_entry_node: Some(self.run_as_entry_node),
            drop_neighbors_on_salt_update: Some(self.drop_neighbors_on_salt_update),
        }
    }
}

// Note: In case someone wonders why we use `Option<bool>`: Although serde actually provides a way to allow for the
// default of a boolean parameter to be `true` - so that missing config parameters could be created on the fly - it felt
// too awkward and also a bit too cumbersome to me: serde(default = "default_providing_function_name").

/// The autopeering config JSON builder.
///
/// Note: Fields will be camel-case formatted.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename = "autopeering")]
pub struct AutopeeringConfigJsonBuilder {
    /// Wether autopeering should be enabled.
    pub enabled: bool,
    /// The bind address for the server.
    #[serde(rename = "bindAddress")]
    pub bind_addr: SocketAddr,
    /// The entry nodes for bootstrapping.
    #[serde(rename = "entryNodes")]
    pub entry_nodes: Vec<AutopeeringMultiaddr>,
    /// Whether `Ipv4` or `Ipv6` should be preferred in case a hostname supports both.
    #[serde(rename = "entryNodesPreferIPv6")]
    pub entry_nodes_prefer_ipv6: Option<bool>,
    /// Whether the node should run as an entry node.
    #[serde(rename = "runAsEntryNode")]
    pub run_as_entry_node: Option<bool>,
    /// Whether all neighbors should be disconnected from when the salts are updated.
    #[serde(rename = "dropNeighborsOnSaltUpdate")]
    pub drop_neighbors_on_salt_update: Option<bool>,
}

impl AutopeeringConfigJsonBuilder {
    /// Builds the actual `AutopeeringConfig`.
    pub fn finish(self) -> AutopeeringConfig {
        AutopeeringConfig {
            enabled: self.enabled,
            bind_addr: self.bind_addr,
            entry_nodes: self.entry_nodes,
            entry_nodes_prefer_ipv6: self.entry_nodes_prefer_ipv6.unwrap_or(ENTRYNODES_PREFER_IPV6_DEFAULT),
            run_as_entry_node: self.run_as_entry_node.unwrap_or(RUN_AS_ENTRYNODE_DEFAULT),
            drop_neighbors_on_salt_update: self
                .drop_neighbors_on_salt_update
                .unwrap_or(DROP_NEIGHBORS_ON_SALT_UPDATE_DEFAULT),
        }
    }
}

impl Default for AutopeeringConfigJsonBuilder {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_addr: SocketAddr::new(AUTOPEERING_BIND_ADDR_DEFAULT, AUTOPEERING_BIND_PORT_DEFAULT),
            entry_nodes: Vec::default(),
            entry_nodes_prefer_ipv6: Some(false),
            run_as_entry_node: Some(false),
            drop_neighbors_on_salt_update: Some(false),
        }
    }
}

/// The autopeering config TOML builder.
///
/// Note: Fields will be snake-case formatted.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename = "autopeering")]
pub struct AutopeeringConfigTomlBuilder {
    /// Wether autopeering should be enabled.
    pub enabled: bool,
    /// The bind address for the server.
    #[serde(rename = "bind_address")]
    pub bind_addr: SocketAddr,
    /// The entry nodes for bootstrapping.
    pub entry_nodes: Vec<AutopeeringMultiaddr>,
    /// Whether `Ipv4` or `Ipv6` should be preferred in case a hostname supports both.
    pub entry_nodes_prefer_ipv6: Option<bool>,
    /// Whether the node should run as an entry node.
    pub run_as_entry_node: Option<bool>,
    /// Whether all neighbors should be disconnected from when the salts are updated.
    pub drop_neighbors_on_salt_update: Option<bool>,
}

impl AutopeeringConfigTomlBuilder {
    /// Builds the actual `AutopeeringConfig`.
    pub fn finish(self) -> AutopeeringConfig {
        AutopeeringConfig {
            enabled: self.enabled,
            bind_addr: self.bind_addr,
            entry_nodes: self.entry_nodes,
            entry_nodes_prefer_ipv6: self.entry_nodes_prefer_ipv6.unwrap_or(ENTRYNODES_PREFER_IPV6_DEFAULT),
            run_as_entry_node: self.run_as_entry_node.unwrap_or(RUN_AS_ENTRYNODE_DEFAULT),
            drop_neighbors_on_salt_update: self
                .drop_neighbors_on_salt_update
                .unwrap_or(DROP_NEIGHBORS_ON_SALT_UPDATE_DEFAULT),
        }
    }
}

impl Default for AutopeeringConfigTomlBuilder {
    fn default() -> Self {
        let json_builder = AutopeeringConfigJsonBuilder::default();
        Self {
            enabled: json_builder.enabled,
            bind_addr: json_builder.bind_addr,
            entry_nodes: json_builder.entry_nodes,
            entry_nodes_prefer_ipv6: json_builder.entry_nodes_prefer_ipv6,
            run_as_entry_node: json_builder.run_as_entry_node,
            drop_neighbors_on_salt_update: json_builder.drop_neighbors_on_salt_update,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    impl fmt::Display for AutopeeringConfigJsonBuilder {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            serde_json::to_string_pretty(self).fmt(f)
        }
    }

    impl fmt::Display for AutopeeringConfigTomlBuilder {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            toml::to_string_pretty(self).fmt(f)
        }
    }

    fn create_json_config_from_str() -> AutopeeringConfigJsonBuilder {
        let config_json_str = r#"
        {
            "enabled": true,
            "bindAddress": "0.0.0.0:15626",
            "entryNodes": [
                "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
                "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
                "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
                "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
            ],
            "entryNodesPreferIPv6": true,
            "runAsEntryNode": false,
            "dropNeighborsOnSaltUpdate": false
        }"#;

        serde_json::from_str(config_json_str).expect("error deserializing json config")
    }

    fn create_toml_config_from_str() -> AutopeeringConfigTomlBuilder {
        let toml_config_str = r#"
            enabled = true
            bind_address = "0.0.0.0:15626"
            entry_nodes = [
                "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM",
                "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
                "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2",
                "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC"
            ]
            entry_nodes_prefer_ipv6 = true
            run_as_entry_node = false
            drop_neighbors_on_salt_update = false
        "#;

        toml::from_str(toml_config_str).unwrap()
    }

    fn create_config() -> AutopeeringConfig {
        AutopeeringConfig {
            enabled: true,
            bind_addr: "0.0.0.0:15626".parse().unwrap(),
            entry_nodes: vec![
                "/dns/lucamoser.ch/udp/14826/autopeering/4H6WV54tB29u8xCcEaMGQMn37LFvM1ynNpp27TTXaqNM".parse().unwrap(),
                "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb".parse().unwrap(),
                "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2".parse().unwrap(),
                "/dns/entry-mainnet.tanglebay.com/udp/14626/autopeering/iot4By1FD4pFLrGJ6AAe7YEeSu9RbW9xnPUmxMdQenC".parse().unwrap(),
            ],
            entry_nodes_prefer_ipv6: true,
            run_as_entry_node: false,
            drop_neighbors_on_salt_update: false,
        }
    }

    /// Tests config serialization and deserialization.
    #[test]
    fn config_serde() {
        // Create format dependent configs from their respective string representation.
        let json_config = create_json_config_from_str();
        let toml_config = create_toml_config_from_str();

        // Manually create an instance of a config.
        let config = create_config();

        // Compare whether the deserialized JSON str equals the JSON-serialized config instance.
        assert_eq!(
            json_config,
            config.clone().into_json_config(),
            "json config de/serialization failed"
        );

        // Compare whether the deserialized TOML str equals the TOML-serialized config instance.
        assert_eq!(
            toml_config,
            config.into_toml_config(),
            "toml config de/serialization failed"
        );
    }
}
