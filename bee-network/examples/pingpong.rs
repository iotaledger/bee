// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This example shows how to create and run 2 TCP nodes using `bee_network`, that will
//! automatically add eachother as peers and exchange the messages 'ping' and 'pong'
//! respectively.
//!
//! You might want to run several instances of such a node in separate
//! terminals and connect those instances by specifying commandline arguments.
//!
//! ```bash
//! cargo r --example pingpong -- --bind /ip4/127.0.0.1/tcp/1337 --peers /ip4/127.0.0.1/tcp/1338 --msg ping
//! cargo r --example pingpong -- --bind /ip4/127.0.0.1/tcp/1338 --peers /ip4/127.0.0.1/tcp/1337 --msg pong
//! ```

#![allow(dead_code, unused_imports)]

mod common;

use common::*;

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{
    node::{Node, NodeBuilder, ResHandle},
    worker::Worker,
};
use bee_network::{Command::*, Event, Keypair, Multiaddr, NetworkConfig, NetworkController, NetworkListener, PeerId};

use async_trait::async_trait;
use futures::{
    channel::oneshot,
    sink::SinkExt,
    stream::{Fuse, StreamExt},
    AsyncWriteExt, Future, FutureExt,
};
use log::*;
use structopt::StructOpt;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    convert::Infallible,
    time::Duration,
};

const RECONNECT_MILLIS: u64 = 5000;

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    let config = args.into_config();

    logger::init(log::LevelFilter::Trace);

    let node = ExampleNodeBuilder::new(config.clone()).unwrap().finish().await.unwrap();

    // let mut network_controller = node.network_controller.clone();

    // info!("[EXAMPLE] Dialing unkown peers (by address)...");
    // for peer_address in &config.peers {
    //     if let Err(e) = network_controller.send(DialAddress {
    //         address: peer_address.clone(),
    //     }) {
    //         warn!("Dialing peer (by address) failed. Error: {:?}", e);
    //     }
    // }

    info!("[EXAMPLE] ...finished.");

    node.run().await;
}

struct ExampleNode {
    config: ExampleConfig,
    network_listener: NetworkListener,
    connected_peers: HashSet<PeerId>,
}

impl ExampleNode {
    async fn run(self) {
        let ExampleNode {
            config,
            network_listener,
            mut connected_peers,
        } = self;

        info!("[EXAMPLE] Node running.");

        let sigterm_listener = ctrl_c_listener();
        let mut network_events = ShutdownStream::new(sigterm_listener, network_listener);

        while let Some(event) = network_events.next().await {
            info!("Received {:?}.", event);

            // process_event(event, &config.message, &mut network_controller, &mut connected_peers).await;
        }

        info!("[EXAMPLE] Stopping node...");
        // if let Err(e) = shutdown.execute().await {
        //     warn!("Sending shutdown signal failed. Error: {:?}", e);
        // }

        info!("[EXAMPLE] Stopped.");
    }
}

#[async_trait]
impl Node for ExampleNode {
    type Builder = ExampleNodeBuilder;
    type Backend = DummyBackend;
    type Error = Infallible;

    async fn stop(mut self) -> Result<(), Self::Error> {
        todo!("stop")
    }

    fn spawn<W, G, F>(&mut self, g: G)
    where
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static,
    {
        todo!("spawn")
    }

    fn worker<W>(&self) -> Option<&W>
    where
        W: Worker<Self> + Send + Sync,
    {
        todo!("worker")
    }

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R) {
        todo!("remove_resource")
    }

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R> {
        todo!("remove_resource")
    }

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResHandle<R> {
        todo!("resource")
    }
}

struct ExampleNodeBuilder {
    config: ExampleConfig,
}

#[async_trait(?Send)]
impl NodeBuilder<ExampleNode> for ExampleNodeBuilder {
    type Error = Infallible;
    type Config = ExampleConfig;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self { config })
    }

    fn with_worker<W: Worker<ExampleNode> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self
    }

    fn with_worker_cfg<W: Worker<ExampleNode> + 'static>(self, config: W::Config) -> Self {
        self
    }

    fn with_resource<R: Any + Send + Sync>(self, res: R) -> Self {
        self
    }

    async fn finish(self) -> Result<ExampleNode, Self::Error> {
        let network_config = NetworkConfig::build()
            .bind_address(&self.config.bind_address.to_string())
            .reconnect_millis(RECONNECT_MILLIS)
            .finish();

        info!("[EXAMPLE] Initializing network...");

        let local_keys = Keypair::generate();
        let network_id = 1;
        let message = self.config.message.clone();

        let (this, network_listener) =
            bee_network::init::<ExampleNode>(network_config, local_keys, network_id, self).await;

        info!("[EXAMPLE] Node initialized.");
        Ok(ExampleNode {
            config: this.config,
            network_listener,
            connected_peers: HashSet::new(),
        })
    }
}

#[inline]
async fn process_event(event: Event, message: &str, network: &mut NetworkController, peers: &mut HashSet<PeerId>) {
    match event {
        Event::PeerConnected { id, address, .. } => {
            info!("[EXAMPLE] Connected peer '{}' with address '{}'.", id, address);

            info!("[EXAMPLE] Sending message: \"{}\"", message);
            if let Err(e) = network.send(SendMessage {
                message: Utf8Message::new(message).as_bytes(),
                to: id.clone(),
            }) {
                warn!("Sending message to peer failed. Error: {:?}", e);
            }

            spam_endpoint(network.clone(), id);
        }

        Event::PeerDisconnected { id } => {
            info!("[EXAMPLE] Disconnected peer {}.", id);
        }

        Event::MessageReceived { message, from } => {
            info!("[EXAMPLE] Received message from {} (length: {}).", from, message.len());

            let message = Utf8Message::from_bytes(&message);
            info!("[EXAMPLE] Received message \"{}\"", message);
        }

        _ => warn!("Unsupported event {:?}.", event),
    }
}

fn ctrl_c_listener() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();

        sender.send(()).unwrap();
    });

    receiver
}

fn spam_endpoint(mut network: NetworkController, peer_id: PeerId) {
    info!("[EXAMPLE] Now sending spam messages to {}", peer_id);

    tokio::spawn(async move {
        for i in 0u64.. {
            tokio::time::delay_for(Duration::from_secs(5)).await;

            let message = Utf8Message::new(&i.to_string());

            if let Err(e) = network.send(SendMessage {
                message: message.as_bytes(),
                to: peer_id.clone(),
            }) {
                warn!("Sending message to peer failed. Error: {:?}", e);
            }
        }
    });
}
