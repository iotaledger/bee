// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering implementation for the Bee framework.

// #![warn(missing_docs)]
#![allow(warnings)]

mod command;
mod delay;
mod discovery;
mod distance;
mod filter;
mod hash;
mod local;
mod multiaddr;
mod packet;
mod peering;
mod peering_messages;
mod peerlist;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}
mod request;
mod ring;
mod salt;
mod server;
mod task;
mod time;

pub mod config;
pub mod event;
pub mod identity;
pub mod init;
pub mod peer;
pub mod peerstore;
pub mod service_map;

pub use config::AutopeeringConfig;
pub use event::Event;
pub use identity::PeerId;
pub use init::init;
pub use local::Local;
pub use peer::Peer;
pub use service_map::{ServiceMap, ServiceName, ServiceTransport};
