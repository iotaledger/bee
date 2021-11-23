// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains the following default node implementations:
//! * A Bee full node;
//! * A Bee entry node (autopeering);

#![deny(missing_docs)]

mod cli;
mod config;
mod core;
mod entrynode;
mod fullnode;
mod local;
mod shutdown;
mod storage;
mod util;

pub mod plugins;
pub mod tools;

pub use cli::ClArgs;
pub use config::{NodeConfig, NodeConfigBuilder};
pub use entrynode::{builder::EntryNodeBuilder, config::EntryNodeConfig, EntryNode};
pub use fullnode::{builder::FullNodeBuilder, config::FullNodeConfig, FullNode};
pub use storage::NodeStorageBackend;
pub use util::print_banner_and_version;

pub(crate) const BEE_NAME: &str = "Bee";
pub(crate) const BEE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BEE_GIT_COMMIT: &str = env!("GIT_COMMIT");
pub(crate) const AUTOPEERING_VERSION: u32 = 1;
pub(crate) const PEERSTORE_PATH: &str = "./peerstore";
pub(crate) const LOCAL_ALIAS_DEFAULT: &str = "bee";
pub(crate) const BECH32_HRP_DEFAULT: &str = "iota";
pub(crate) const NETWORK_NAME_DEFAULT: &str = "iota";
pub(crate) const KEYPAIR_STR_LENGTH: usize = 128;
