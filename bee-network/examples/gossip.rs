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
//! cargo r --example gossip -- --bind /ip4/127.0.0.1/tcp/1337 --peers /ip4/127.0.0.1/tcp/1338
//! cargo r --example gossip -- --bind /ip4/127.0.0.1/tcp/1338 --peers /ip4/127.0.0.1/tcp/1337
//! ```

#![allow(dead_code, unused_imports)]

mod common;

use common::*;

use bee_network::{
    Command::*, Event, Keypair, Multiaddr, NetworkConfig, NetworkController, NetworkListener, PeerId, PeerInfo,
    PeerRelation,
};
use bee_runtime::{
    node::{Node, NodeBuilder},
    resource::ResourceHandle,
    shutdown_stream::ShutdownStream,
    worker::Worker,
};

use anymap::{any::Any as AnyMapAny, Map};
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
    any::{type_name, Any, TypeId},
    collections::{HashMap, HashSet},
    convert::Infallible,
    pin::Pin,
    time::Duration,
};

const RECONNECT_INTERVAL_SECS: u64 = 30;

type WorkerStart<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
type WorkerStop<N> = dyn for<'a> FnOnce(&'a mut N) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> + Send;
type ResourceRegister<N> = dyn for<'a> FnOnce(&'a mut N);

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::from_args();
    let config = args.into_config();

    logger::init(log::LevelFilter::Info);

    let node = ExampleNodeBuilder::new(config.clone()).unwrap().finish().await.unwrap();

    let network_controller = node.resource::<NetworkController>();

    info!("[EXAMPLE] Dialing unkown peers (by address)...");
    for peer_address in &config.peers {
        if let Err(e) = network_controller.send(DialAddress {
            address: peer_address.clone(),
        }) {
            warn!("Dialing peer (by address) failed. Error: {:?}", e);
        }
    }

    info!("[EXAMPLE] ...finished.");

    node.run().await;
}

struct ExampleNode {
    config: ExampleConfig,
    network_listener: NetworkListener,
    connected_peers: HashMap<PeerId, oneshot::Sender<()>>,
    workers: Map<dyn AnyMapAny + Send + Sync>,
    tasks: HashMap<
        TypeId,
        Vec<(
            oneshot::Sender<()>,
            // TODO Result ?
            Box<dyn Future<Output = Result<(), tokio::task::JoinError>> + Send + Sync + Unpin>,
        )>,
    >,
    resources: Map<dyn AnyMapAny + Send + Sync>,
}

impl ExampleNode {
    async fn run(self) {
        let network_controller = self.resource::<NetworkController>();

        let ExampleNode {
            network_listener,
            mut connected_peers,
            ..
        } = self;

        info!("[EXAMPLE] Node running.");

        let sigterm_listener = ctrl_c_listener();
        let mut network_events = ShutdownStream::new(sigterm_listener, network_listener);

        while let Some(event) = network_events.next().await {
            info!("Received {:?}.", event);

            process_event(event, &network_controller, &mut connected_peers).await;
        }

        info!("[EXAMPLE] Stopping node...");

        for (_, shutdown_tx) in connected_peers {
            shutdown_tx.send(()).expect("Sending shutdown signal failed");
        }

        info!("[EXAMPLE] Stopped.");
    }

    fn add_worker<W: Worker<Self> + Send + Sync>(&mut self, worker: W) {
        self.workers.insert(worker);
    }

    fn remove_worker<W: Worker<Self> + Send + Sync>(&mut self) -> W {
        self.workers
            .remove()
            .unwrap_or_else(|| panic!("Failed to remove worker `{}`", type_name::<W>()))
    }
}

#[async_trait]
impl Node for ExampleNode {
    type Builder = ExampleNodeBuilder;
    type Backend = DummyBackend;
    type Error = Infallible;

    async fn stop(mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn register_resource<R: Any + Send + Sync>(&mut self, res: R) {
        self.resources.insert(ResourceHandle::new(res));
    }

    fn remove_resource<R: Any + Send + Sync>(&mut self) -> Option<R> {
        self.resources.remove::<ResourceHandle<R>>()?.try_unwrap()
    }

    #[track_caller]
    fn resource<R: Any + Send + Sync>(&self) -> ResourceHandle<R> {
        self.resources
            .get::<ResourceHandle<R>>()
            .unwrap_or_else(|| panic!("Unable to fetch node resource {}", type_name::<R>()))
            .clone()
    }

    fn spawn<W, G, F>(&mut self, g: G)
    where
        W: Worker<Self>,
        G: FnOnce(oneshot::Receiver<()>) -> F,
        F: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        self.tasks
            .entry(TypeId::of::<W>())
            .or_default()
            .push((tx, Box::new(tokio::spawn(g(rx)))));
    }

    fn worker<W>(&self) -> Option<&W>
    where
        W: Worker<Self> + Send + Sync,
    {
        self.workers.get::<W>()
    }
}

struct ExampleNodeBuilder {
    config: ExampleConfig,
    deps: HashMap<TypeId, &'static [TypeId]>,
    worker_starts: HashMap<TypeId, Box<WorkerStart<ExampleNode>>>,
    worker_stops: HashMap<TypeId, Box<WorkerStop<ExampleNode>>>,
    resource_registers: Vec<Box<ResourceRegister<ExampleNode>>>,
}

#[async_trait(?Send)]
impl NodeBuilder<ExampleNode> for ExampleNodeBuilder {
    type Error = Infallible;
    type Config = ExampleConfig;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            config,
            deps: HashMap::default(),
            worker_starts: HashMap::default(),
            worker_stops: HashMap::default(),
            resource_registers: Vec::default(),
        })
    }

    fn with_worker<W: Worker<ExampleNode> + 'static>(self) -> Self
    where
        W::Config: Default,
    {
        self.with_worker_cfg::<W>(W::Config::default())
    }

    fn with_worker_cfg<W: Worker<ExampleNode> + 'static>(mut self, config: W::Config) -> Self {
        self.worker_starts.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    debug!("Starting worker {}...", type_name::<W>());
                    match W::start(node, config).await {
                        Ok(w) => node.add_worker(w),
                        Err(e) => panic!("Worker `{}` failed to start: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self.worker_stops.insert(
            TypeId::of::<W>(),
            Box::new(|node| {
                Box::pin(async move {
                    debug!("Stopping worker {}...", type_name::<W>());
                    match node.remove_worker::<W>().stop(node).await {
                        Ok(()) => {}
                        Err(e) => panic!("Worker `{}` failed to stop: {:?}.", type_name::<W>(), e),
                    }
                })
            }),
        );
        self
    }

    fn with_resource<R: Any + Send + Sync>(mut self, res: R) -> Self {
        self.resource_registers.push(Box::new(move |node| {
            node.register_resource(res);
        }));
        self
    }

    async fn finish(self) -> Result<ExampleNode, Self::Error> {
        let network_config = NetworkConfig::build()
            .bind_address(&self.config.bind_address.to_string())
            .reconnect_interval_secs(RECONNECT_INTERVAL_SECS)
            .finish();

        info!("[EXAMPLE] Initializing network...");

        let local_keys = Keypair::generate();
        let network_id = 1;

        let (mut this, network_listener) =
            bee_network::init::<ExampleNode>(network_config, local_keys, network_id, 1, self).await;

        info!("[EXAMPLE] Node initialized.");

        let mut node = ExampleNode {
            config: this.config,
            network_listener,
            connected_peers: HashMap::new(),
            resources: Map::new(),
            tasks: HashMap::new(),
            workers: Map::new(),
        };

        for f in this.resource_registers {
            f(&mut node);
        }

        for (_, f) in this.worker_starts.drain() {
            f(&mut node).await
        }

        Ok(node)
    }
}

#[inline]
async fn process_event(
    event: Event,
    network: &NetworkController,
    connected_peers: &mut HashMap<PeerId, oneshot::Sender<()>>,
) {
    match event {
        Event::PeerAdded { id, info } => {
            info!("[EXAMPLE] Added peer '{}' ({:?}).", id, info);
        }
        Event::PeerConnected { id, address, .. } => {
            info!("[EXAMPLE] Connected peer '{}' with address '{}'.", id, address);

            let (tx, rx) = oneshot::channel::<()>();

            connected_peers.insert(id.clone(), tx);

            simulate_gossip(network.clone(), id, rx);
        }

        Event::PeerDisconnected { id } => {
            // Removing the peer will trigger the shutdown signal, which stops the gossip task
            connected_peers.remove(&id);

            info!("[EXAMPLE] Disconnected peer {}.", id);
        }

        Event::MessageReceived { message, from } => {
            info!("[EXAMPLE] Received message from {} (length: {}).", from, message.len());

            let message = Utf8Message::from_bytes(&message);
            info!("[EXAMPLE] Received message \"{}\"", message);
        }

        // Ignore all other events
        _ => (),
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

fn simulate_gossip(network: NetworkController, peer_id: PeerId, shutdown_rx: oneshot::Receiver<()>) {
    info!("[EXAMPLE] Simulating gossip with {}", peer_id);

    tokio::spawn(async move {
        let mut shutdown_stream = ShutdownStream::new(shutdown_rx, tokio::time::interval(Duration::from_secs(5)));

        while let Some(_) = shutdown_stream.next().await {
            let message = Utf8Message::new(&i.to_string());

            info!("[EXAMPLE] Sending message: \"{}\"", message);
            if let Err(e) = network.send(SendMessage {
                message: message.as_bytes(),
                to: peer_id.clone(),
            }) {
                warn!("Sending message to peer failed. Error: {:?}", e);
            }

            i += 1;
        }

        info!("[EXAMPLE] Stopping gossip with {}", peer_id);
    });
}
