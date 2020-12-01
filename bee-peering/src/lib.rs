// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::module_inception)]

mod config;
mod manager;
mod manual;

pub use config::{PeeringConfig, PeeringConfigBuilder};
pub use manager::PeerManager;
pub use manual::ManualPeerManager;
