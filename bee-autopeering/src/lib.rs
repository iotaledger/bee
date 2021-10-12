// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering implementation for the Bee framework.

#![warn(missing_docs)]

mod config;
mod discovery;
mod distance;
mod hash;
mod identity;
mod init;
mod messages;
mod multiaddr;
mod packets;
mod peer;
mod peering;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}
mod salt;
mod server;
mod store;
mod time;

pub use init::init;
