// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The central implementation crate of IOTA's Bee node.

#![allow(clippy::unit_arg)]
#![deny(missing_docs)]

mod banner;
mod cli;
mod config;
mod constants;
mod node;
mod storage;

/// Plugin support traits/types and officially supported plugins such the MQTT and dashboard plugins.
pub mod plugins;
/// Utilities and tools that may be invoked via the node binary.
pub mod tools;

pub use banner::print_banner_and_version;
pub use cli::CliArgs;
pub use config::NodeConfigBuilder;
pub use node::{BeeNode as Node, BeeNodeBuilder as NodeBuilder, Error};
