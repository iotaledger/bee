// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains the default node implementations of the Bee framework:
//! * Bee full node;
//! * Bee (autopeering) entry node;

#![allow(warnings)]

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

pub use cli::CliArgs;
pub use config::{NodeConfig, NodeConfigBuilder};
pub use entrynode::{builder::EntryNodeBuilder, config::EntryNodeConfig, EntryNode};
pub use fullnode::{builder::FullNodeBuilder, FullNode, config::FullNodeConfig};
pub use util::print_banner_and_version;

use futures::future::Future;

use std::pin::Pin;

pub(crate) const BEE_NAME: &str = "Bee";
pub(crate) const BEE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BEE_GIT_COMMIT: &str = env!("GIT_COMMIT");
pub(crate) const AUTOPEERING_VERSION: u32 = 1;
pub(crate) const PEERSTORE_PATH: &str = "./peerstore";
pub(crate) const LOCAL_ALIAS_DEFAULT: &str = "bee";
pub(crate) const BECH32_HRP_DEFAULT: &str = "iota";
pub(crate) const NETWORK_NAME_DEFAULT: &str = "iota";
pub(crate) const KEYPAIR_STR_LENGTH: usize = 128;
