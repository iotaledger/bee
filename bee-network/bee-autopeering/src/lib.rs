// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Allows peers in the same IOTA network to automatically discover each other.
//!
//! In order to integrate the Autopeering functionality in your node implementation you need to provide its `init`
//! function with the following data:
//! * an `AutopeeringConfig`;
//! * a protocol version (`u32`);
//! * a network name, e.g. "chrysalis-mainnet";
//! * a `Local` entity (either randomly created or from an `Ed25519` keypair), that additionally announces one or more
//!   services;
//! * a shutdown signal (`Future`);
//! * a peer store, e.g. the `InMemoryPeerStore` (non-persistent) or the `SledPeerStore` (persistent), or a custom peer
//!   store implementing the `PeerStore` trait;
//!
//! ## Example
//!
//! ```no_run
//! use bee_autopeering::{
//!     config::AutopeeringConfigJsonBuilder,
//!     init,
//!     stores::{SledPeerStore, SledPeerStoreConfig},
//!     AutopeeringConfig, Event, Local, NeighborValidator, Peer, ServiceProtocol, AUTOPEERING_SERVICE_NAME,
//! };
//!
//! const NETWORK: &str = "chrysalis-mainnet";
//!
//! // An example autopeering config in JSON format:
//! fn read_config() -> AutopeeringConfig {
//!     let config_json = r#"
//!     {
//!         "enabled": true,
//!         "bindAddress": "0.0.0.0:14627",
//!         "entryNodes": [
//!             "/dns/entry-hornet-0.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaPHdAn7eueBnXtikZMwhfPXaeGJGXDt4RBuLuGgb",
//!             "/dns/entry-hornet-1.h.chrysalis-mainnet.iotaledger.net/udp/14626/autopeering/iotaJJqMd5CQvv1A61coSQCYW9PNT1QKPs7xh2Qg5K2"
//!         ],
//!         "entryNodesPreferIPv6": false,
//!         "runAsEntryNode": false
//!     }"#;
//!
//!     serde_json::from_str::<AutopeeringConfigJsonBuilder>(config_json)
//!         .expect("error deserializing json config builder")
//!         .finish()
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Peers will only accept each other as peer if they agree on the protocol version and the
//!     // network name.
//!     const VERSION: u32 = 1;
//!
//!     // Read the config from a JSON file/string (TOML is also supported).
//!     let config = read_config();
//!
//!     // Create a random local entity, that announces two services:
//!     let local = {
//!         let l = Local::generate();
//!
//!         l.add_service(
//!             AUTOPEERING_SERVICE_NAME,
//!             ServiceProtocol::Udp,
//!             config.bind_addr().port(),
//!         );
//!         l.add_service(NETWORK, ServiceProtocol::Tcp, 15600);
//!         l
//!     };
//!
//!     // You can choose between the `InMemoryPeerStore` (non-persistent), the `SledPeerStore`
//!     // (persistent), or your own implementation that implements the `PeerStore` trait.
//!     let peer_store_config = SledPeerStoreConfig::new().path("./peerstore");
//!
//!     // The `NeighborValidator` allows you to accept only certain peers as neighbors, e.g. only those
//!     // with enabled Gossip service.
//!     let neighbor_validator = GossipNeighborValidator {};
//!
//!     // You need to provide some form of shutdown signal (any `Future` impl is allowed).
//!     let term_signal = tokio::signal::ctrl_c();
//!
//!     // With initializing the autopeering system you receive an event stream receiver.
//!     let mut event_rx = bee_autopeering::init::<SledPeerStore, _, _, GossipNeighborValidator>(
//!         config.clone(),
//!         VERSION,
//!         NETWORK,
//!         local,
//!         peer_store_config,
//!         term_signal,
//!         neighbor_validator,
//!     )
//!     .await
//!     .expect("initializing autopeering system failed");
//!
//!     // You can then process autopeering events.
//!     loop {
//!         tokio::select! {
//!             e = event_rx.recv() => {
//!                 if let Some(event) = e {
//!                     // handle the event
//!                     // process(event);
//!                 } else {
//!                     break;
//!                 }
//!             }
//!         };
//!     }
//! }
//!
//! #[derive(Clone)]
//! struct GossipNeighborValidator {}
//!
//! impl NeighborValidator for GossipNeighborValidator {
//!     fn is_valid(&self, peer: &Peer) -> bool {
//!         peer.has_service(NETWORK)
//!     }
//! }
//! ```

#![deny(missing_docs)]

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
    services::{ServiceEndpoint, ServiceMap, ServiceName, ServiceProtocol, AUTOPEERING_SERVICE_NAME},
    Local,
};
pub use peer::{peer_id, peer_id::PeerId, stores, Peer};
pub use peering::{Distance, NeighborValidator, Status};
