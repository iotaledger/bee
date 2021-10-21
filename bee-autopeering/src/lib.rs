// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering implementation for the Bee framework.

// #![warn(missing_docs)]
#![allow(warnings)]

mod delay;
mod discovery;
mod discovery_messages;
mod distance;
mod filter;
mod hash;
mod local;
mod multiaddr;
mod packet;
mod peering;
mod peering_messages;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}
mod request;
mod salt;
mod server;
mod shutdown;
mod time;

pub mod config;
pub mod identity;
pub mod init;
pub mod peer;
pub mod peerstore;
pub mod service_map;

pub use config::AutopeeringConfig;
pub use discovery::DiscoveryEvent;
pub use identity::PeerId;
pub use init::init;
pub use local::Local;
pub use peer::Peer;
pub use peering::PeeringEvent;
pub use service_map::{ServiceMap, ServiceName, ServiceTransport};
