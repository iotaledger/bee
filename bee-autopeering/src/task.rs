// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{peer::peerlist::ActivePeersList, time::SECOND};

use priority_queue::PriorityQueue;
use tokio::{sync::oneshot, task::JoinHandle, time};

use std::{
    any::Any,
    collections::HashMap,
    future::Future,
    hash::Hasher,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

pub(crate) const MAX_SHUTDOWN_PRIORITY: u8 = 255;
pub(crate) const MIN_SHUTDOWN_PRIORITY: u8 = 0;
const SHUTDOWN_TIMEOUT_SECS: Duration = Duration::from_secs(5 * SECOND);

pub(crate) type ShutdownRx = oneshot::Receiver<()>;
type ShutdownTx = oneshot::Sender<()>;

pub(crate) type Repeat<T: Send + Sync> = Box<dyn for<'a> Fn(&'a T) + Send>;

/// A type implementing `Runnable` executes code whenever
#[async_trait::async_trait]
pub(crate) trait Runnable {
    const NAME: &'static str;
    const SHUTDOWN_PRIORITY: u8;

    type ShutdownSignal: Future + Send + Unpin + 'static;

    async fn run(self, shutdown_rx: Self::ShutdownSignal);
}

pub(crate) struct TaskManager<const N: usize> {
    shutdown_handles: HashMap<String, JoinHandle<()>>,
    shutdown_order: PriorityQueue<String, u8>,
    shutdown_senders: HashMap<String, ShutdownTx>,
    shutdown_fn: HashMap<String, Box<dyn Any + Send>>,
}

impl<const N: usize> TaskManager<N> {
    pub(crate) fn new() -> Self {
        Self {
            shutdown_order: PriorityQueue::with_capacity(N),
            shutdown_senders: HashMap::with_capacity(N),
            shutdown_handles: HashMap::with_capacity(N),
            shutdown_fn: HashMap::default(),
        }
    }

    pub(crate) fn run<R>(&mut self, runnable: R)
    where
        R: Runnable<ShutdownSignal = ShutdownRx> + 'static,
    {
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_senders.insert(R::NAME.into(), shutdown_tx);

        let handle = tokio::spawn(runnable.run(shutdown_rx));
        log::info!("`{}` running.", R::NAME);

        assert!(!self.shutdown_handles.contains_key(R::NAME));
        self.shutdown_handles.insert(R::NAME.into(), handle);

        self.shutdown_order.push(R::NAME.into(), R::SHUTDOWN_PRIORITY);
    }

    pub(crate) fn shutdown_rx(&mut self, name: &str, priority: u8) -> ShutdownRx {
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_senders.insert(name.into(), shutdown_tx);

        self.shutdown_order.push(name.into(), priority);

        shutdown_rx
    }

    pub(crate) fn repeat<T, D>(&mut self, cmd: Repeat<T>, mut delay: D, ctx: T, name: &str, shutdown_priority: u8)
    where
        T: Send + Sync + 'static,
        D: Iterator<Item = Duration> + Send + 'static,
    {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_senders.insert(name.into(), shutdown_tx);

        let handle = tokio::spawn(async move {
            while let Some(duration) = delay.next() {
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    _ = time::sleep(duration) => cmd(&ctx),
                }
            }
        });
        log::info!("`{}` repeating.", name);

        assert!(!self.shutdown_handles.contains_key(name));
        self.shutdown_handles.insert(name.into(), handle);

        self.shutdown_order.push(name.into(), shutdown_priority);
    }

    pub(crate) fn add_shutdown_fn(&mut self, b: Box<dyn Fn(&ActivePeersList)>) {
        todo!()
    }

    pub(crate) async fn shutdown(self) {
        let TaskManager {
            mut shutdown_order,
            shutdown_handles: mut runnable_handles,
            mut shutdown_senders,
            shutdown_fn,
        } = self;

        // Send the shutdown signal to all receivers.
        // TODO: clone necessary?
        let mut shutdown_order_clone = shutdown_order.clone();
        while let Some((task_name, prio)) = shutdown_order_clone.pop() {
            let shutdown_tx = shutdown_senders.remove(&task_name).unwrap();

            log::debug!("Shutting down: {}", task_name);
            shutdown_tx.send(()).expect("error sending shutdown signal");
        }

        // Wait for all tasks to shutdown down in a certain order and maximum amount of time.
        time::timeout(SHUTDOWN_TIMEOUT_SECS, async {
            while let Some((task_name, prio)) = shutdown_order.pop() {
                // Panic: unwrapping is fine, because we are in control of the data.
                let task_handle = runnable_handles.remove(&task_name).unwrap();

                match task_handle.await {
                    Ok(_) => {
                        log::debug!("`{}` stopped.", task_name);
                    }
                    Err(e) => {
                        log::error!("Error shutting down `{}`. Cause: {}", task_name, e);
                    }
                }
            }
        })
        .await;

        log::debug!("Dumping data to storage...");
        for (type_name, f) in shutdown_fn {
            // TODO: flush data into storage
        }
        log::debug!("Finished.");
    }
}
