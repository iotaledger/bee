// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Autopeering implementation for the Bee framework.

// #![warn(missing_docs)]
#![allow(warnings)]

mod command;
mod delay;
mod discovery;
mod hash;
mod local;
mod multiaddr;
mod packet;
mod peer;
mod peering;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}
mod request;
mod server;
mod task;
mod time;

pub mod config;
pub mod event;
pub mod init;

pub use config::AutopeeringConfig;
pub use event::Event;
pub use init::init;
pub use local::{
    service_map::{ServiceMap, ServiceName, ServiceTransport},
    Local,
};
pub use peer::{peer_id, peer_id::PeerId, peerstore, Peer};
pub use peering::{Distance, NeighborValidator, Status};
