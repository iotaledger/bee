// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains the following default node implementations:
//! * A Bee full node;
//! * A Bee entry node (autopeering);

mod cli;
mod config;
mod core;
mod entrynode;
mod fullnode;
mod identity;
mod local;
mod shutdown;
mod storage;
mod util;
mod workers;

pub mod tools;
#[cfg(feature = "trace")]
pub mod trace;

pub use self::{
    cli::ClArgs,
    config::{NodeConfig, NodeConfigBuilder},
    entrynode::{builder::EntryNodeBuilder, config::EntryNodeConfig, EntryNode},
    fullnode::{builder::FullNodeBuilder, config::FullNodeConfig, FullNode},
    identity::{read_keypair_from_pem_file, write_keypair_to_pem_file, PemFileError},
    local::Local,
    storage::NodeStorageBackend,
    util::print_banner_and_version,
};

pub(crate) const BEE_NAME: &str = "Bee";
pub(crate) const BEE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BEE_GIT_COMMIT: &str = env!("GIT_COMMIT");
pub(crate) const AUTOPEERING_VERSION: u32 = 1;
pub(crate) const BECH32_HRP_DEFAULT: &str = "iota";
pub(crate) const NETWORK_NAME_DEFAULT: &str = "iota";
