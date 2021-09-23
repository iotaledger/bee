// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module providing a definition for the node CLI arguments.

use log::LevelFilter;
use structopt::StructOpt;

/// Defines the node CLI arguments.
#[derive(Clone, Debug, StructOpt)]
pub struct NodeCliArgs {
    #[structopt(short = "k", long = "private-key", help = "The node's private key to sign messages")]
    private_key: Option<String>,
    #[structopt(
        long = "gossip-bind-address",
        help = "The bind address of the gossip server, e.g. 127.0.0.1:14666"
    )]
    gossip_bind_addr: Option<String>,
    #[structopt(short = "c", long = "config", help = "Path of the node configuration file")]
    config: Option<String>,
    #[structopt(
        short = "l",
        long = "log-level",
        help = "Stdout log level amongst \"trace\", \"debug\", \"info\", \"warn\" and \"error\""
    )]
    pub(crate) log_level: Option<LevelFilter>,
    #[structopt(short = "v", long = "version", help = "Prints bee version")]
    version: bool,
}

impl Default for NodeCliArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeCliArgs {
    /// Creates a new [`NodeCliArgs`].
    pub fn new() -> Self {
        Self::from_args()
    }

    /// Returns the configuration of the [`NodeCliArgs`].
    pub fn config(&self) -> Option<&str> {
        self.config.as_ref().map(AsRef::as_ref)
    }

    /// Returns the stdout logger level of the [`NodeCliArgs`].
    pub fn log_level(&self) -> Option<&LevelFilter> {
        self.log_level.as_ref()
    }

    /// Returns the version flag of the [`NodeCliArgs`].
    pub fn version(&self) -> bool {
        self.version
    }

    /// Returns the listening port for gossip.
    pub fn gossip_bind_addr(&self) -> Option<&String> {
        self.gossip_bind_addr.as_ref()
    }

    /// Returns the private key of the node.
    pub fn private_key(&self) -> Option<&String> {
        self.private_key.as_ref()
    }
}
