// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering implementation for the Bee framework.

#![warn(missing_docs)]

mod backoff;
mod config;
mod discovery;
mod discovery_messages;
mod distance;
mod hash;
mod identity;
mod init;
mod multiaddr;
mod packet;
mod peer;
mod peering;
mod peering_messages;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}
mod salt;
mod server;
mod service_map;
mod store;
mod time;

pub use identity::LocalId;
pub use init::init;
