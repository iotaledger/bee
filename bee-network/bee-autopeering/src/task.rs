// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{
        lists::{ActivePeersList, ReplacementList},
        PeerStore,
    },
    time::SECOND,
};

use priority_queue::PriorityQueue;
use tokio::{sync::oneshot, task::JoinHandle, time};

use std::{collections::HashMap, future::Future, time::Duration};

pub(crate) const MAX_SHUTDOWN_PRIORITY: u8 = 255;
const SHUTDOWN_TIMEOUT_SECS: Duration = Duration::from_secs(5 * SECOND);

pub(crate) type ShutdownRx = oneshot::Receiver<()>;
type ShutdownTx = oneshot::Sender<()>;

pub(crate) type Repeat<T> = Box<dyn for<'a> Fn(&'a T) + Send>;

/// Represents types driving an event loop.
#[async_trait::async_trait]
pub(crate) trait Runnable {
    const NAME: &'static str;
    const SHUTDOWN_PRIORITY: u8;

    type ShutdownSignal: Future + Send + Unpin + 'static;

    async fn run(self, shutdown_rx: Self::ShutdownSignal);
}

pub(crate) struct TaskManager<S: PeerStore, const N: usize> {
    shutdown_handles: HashMap<String, JoinHandle<()>>,
    shutdown_order: PriorityQueue<String, u8>,
    shutdown_senders: HashMap<String, ShutdownTx>,
    peerstore: S,
    active_peers: ActivePeersList,
    replacements: ReplacementList,
}

impl<S: PeerStore, const N: usize> TaskManager<S, N> {
    pub(crate) fn new(peerstore: S, active_peers: ActivePeersList, replacements: ReplacementList) -> Self {
        Self {
            shutdown_order: PriorityQueue::with_capacity(N),
            shutdown_senders: HashMap::with_capacity(N),
            shutdown_handles: HashMap::with_capacity(N),
            peerstore,
            active_peers,
            replacements,
        }
    }

    /// Runs a `Runnable`, which is a type that features an event loop.
    pub(crate) fn run<R>(&mut self, runnable: R)
    where
        R: Runnable<ShutdownSignal = ShutdownRx> + 'static,
    {
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_senders.insert(R::NAME.into(), shutdown_tx);

        let handle = tokio::spawn(runnable.run(shutdown_rx));
        log::trace!("`{}` running.", R::NAME);

        assert!(!self.shutdown_handles.contains_key(R::NAME));
        self.shutdown_handles.insert(R::NAME.into(), handle);

        self.shutdown_order.push(R::NAME.into(), R::SHUTDOWN_PRIORITY);
    }

    /// Repeats a command in certain intervals provided a context `T`. Will be shut down gracefully with the rest of
    /// all spawned tasks by specifying a `name` and a `shutdown_priority`.
    pub(crate) fn repeat<T, D>(&mut self, cmd: Repeat<T>, mut delay: D, ctx: T, name: &str, shutdown_priority: u8)
    where
        T: Send + Sync + 'static,
        D: Iterator<Item = Duration> + Send + 'static,
    {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_senders.insert(name.into(), shutdown_tx);

        let handle = tokio::spawn(async move {
            for duration in &mut delay {
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    _ = time::sleep(duration) => cmd(&ctx),
                }
            }
        });
        log::trace!("`{}` repeating.", name);

        assert!(!self.shutdown_handles.contains_key(name));
        self.shutdown_handles.insert(name.into(), handle);

        self.shutdown_order.push(name.into(), shutdown_priority);
    }

    /// Executes the system shutdown.
    pub(crate) async fn shutdown(self) {
        let TaskManager {
            mut shutdown_order,
            shutdown_handles: mut runnable_handles,
            mut shutdown_senders,
            peerstore,
            active_peers,
            replacements,
        } = self;

        // Send the shutdown signal to all receivers.
        // TODO: clone necessary?
        let mut shutdown_order_clone = shutdown_order.clone();
        while let Some((task_name, _)) = shutdown_order_clone.pop() {
            // Panic: unwrapping is fine since for every entry in `shutdown_order` there's
            // a corresponding entry in `shutdown_senders`.
            let shutdown_tx = shutdown_senders.remove(&task_name).unwrap();

            log::trace!("Shutting down: {}", task_name);
            shutdown_tx.send(()).expect("error sending shutdown signal");
        }

        // Wait for all tasks to shutdown down in a certain order and maximum amount of time.
        time::timeout(SHUTDOWN_TIMEOUT_SECS, async {
            while let Some((task_name, _)) = shutdown_order.pop() {
                // Panic: unwrapping is fine, because we are in control of the data.
                let task_handle = runnable_handles.remove(&task_name).unwrap();

                match task_handle.await {
                    Ok(_) => {
                        log::trace!("`{}` stopped.", task_name);
                    }
                    Err(e) => {
                        log::error!("Error shutting down `{}`. Cause: {}", task_name, e);
                    }
                }
            }
        })
        .await
        .expect("error awaiting shutdown");

        log::debug!("Flushing data to peerstore...");

        peerstore.delete_all();
        peerstore.store_all_active(&active_peers);
        peerstore.store_all_replacements(&replacements);

        log::debug!("Done.");
    }
}
