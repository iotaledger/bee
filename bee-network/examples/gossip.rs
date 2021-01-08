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
//! cargo r --example gossip -- --bind /ip4/127.0.0.1/tcp/1337 --peers /ip4/127.0.0.1/tcp/1338 --msg hello
//! cargo r --example gossip -- --bind /ip4/127.0.0.1/tcp/1338 --peers /ip4/127.0.0.1/tcp/1337 --msg world
//! ```

#![allow(dead_code, unused_imports)]

mod common;

use common::*;

use bee_network::{Command::*, Event, Keypair, Multiaddr, NetworkConfig, NetworkController, NetworkListener, PeerId};
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

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    let config = args.into_config();

    logger::init(log::LevelFilter::Trace);

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

#[allow(clippy::type_complexity)]
struct ExampleNode {
    config: ExampleConfig,
    network_listener: NetworkListener,
    connected_peers: HashSet<PeerId>,
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
            config,
            network_listener,
            mut connected_peers,
            ..
        } = self;

        info!("[EXAMPLE] Node running.");

        let sigterm_listener = ctrl_c_listener();
        let mut network_events = ShutdownStream::new(sigterm_listener, network_listener);

        while let Some(event) = network_events.next().await {
            info!("Received {:?}.", event);

            process_event(event, &config.message, &network_controller, &mut connected_peers).await;
        }

        info!("[EXAMPLE] Stopping node...");
        // if let Err(e) = shutdown.execute().await {
        //     warn!("Sending shutdown signal failed. Error: {:?}", e);
        // }

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
        // for worker_id in self.worker_order.clone().into_iter().rev() {
        //     for (shutdown, task_fut) in self.tasks.remove(&worker_id).unwrap_or_default() {
        //         let _ = shutdown.send(());
        //         // TODO: Should we handle this error?
        //         let _ = task_fut.await; //.map_err(|e| shutdown::Error::from(worker::Error(Box::new(e))))?;
        //     }
        //     self.worker_stops.remove(&worker_id).unwrap()(&mut self).await;
        //     self.resource::<Bus>().remove_listeners_by_id(worker_id);
        // }

        // // Unwrapping is fine since the node register the backend itself.
        // self.remove_resource::<B>()
        //     .unwrap()
        //     .shutdown()
        //     .await
        //     .map_err(Error::StorageBackend)?;

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

        let (this, network_listener) =
            bee_network::init::<ExampleNode>(network_config, local_keys, network_id, 0, self).await;

        info!("[EXAMPLE] Node initialized.");

        Ok(ExampleNode {
            config: this.config,
            network_listener,
            connected_peers: HashSet::new(),
            resources: Map::new(),
            tasks: HashMap::new(),
            workers: Map::new(),
        })
    }
}

#[inline]
async fn process_event(event: Event, message: &str, network: &NetworkController, _peers: &mut HashSet<PeerId>) {
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

fn spam_endpoint(network: NetworkController, peer_id: PeerId) {
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
