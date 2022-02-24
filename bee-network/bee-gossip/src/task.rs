// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use futures::channel::oneshot;
use priority_queue::PriorityQueue;
use tokio::{task::JoinHandle, time};

use std::{collections::HashMap, future::Future, time::Duration};

pub(crate) const MAX_SHUTDOWN_PRIORITY: u8 = 255;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

type ShutdownTx = oneshot::Sender<()>;
pub(crate) type ShutdownRx = oneshot::Receiver<()>;

pub(crate) fn shutdown_chan() -> (oneshot::Sender<()>, oneshot::Receiver<()>) {
    oneshot::channel::<()>()
}

pub(crate) type Repeat<T> = Box<dyn for<'a> Fn(&'a T) + Send>;

/// Represents types driving an event loop.
#[async_trait::async_trait]
pub(crate) trait Runnable {
    const NAME: &'static str;
    const SHUTDOWN_PRIORITY: u8;

    type ShutdownSignal: Future + Send + Unpin + 'static;

    async fn run(self, shutdown_rx: Self::ShutdownSignal);
}

pub(crate) struct TaskManager {
    shutdown_handles: HashMap<String, JoinHandle<()>>,
    shutdown_senders: HashMap<String, ShutdownTx>,
    shutdown_order: PriorityQueue<String, u8>,
}

impl TaskManager {
    pub(crate) fn new() -> Self {
        Self {
            shutdown_handles: HashMap::default(),
            shutdown_senders: HashMap::default(),
            shutdown_order: PriorityQueue::default(),
        }
    }

    // FIXME:
    /// Runs a `Runnable`, which is a type that features an event loop.
    #[allow(dead_code)]
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
    pub(crate) fn repeat<T, D>(&mut self, f: Repeat<T>, mut delay: D, ctx: T, name: &str, shutdown_priority: u8)
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
                    _ = time::sleep(duration) => f(&ctx),
                }
            }
        });
        log::trace!("`{}` repeating.", name);

        assert!(!self.shutdown_handles.contains_key(name));
        self.shutdown_handles.insert(name.into(), handle);

        self.shutdown_order.push(name.into(), shutdown_priority);
    }

    /// Executes the system shutdown.
    pub(crate) async fn shutdown(self) -> Result<(), ()> {
        let TaskManager {
            mut shutdown_order,
            mut shutdown_handles,
            mut shutdown_senders,
        } = self;

        // Send the shutdown signal to all receivers.
        let mut shutdown_order_clone = shutdown_order.clone();
        while let Some((task_name, _)) = shutdown_order_clone.pop() {
            // Panic: unwrapping is fine since for every entry in `shutdown_order` there's
            // a corresponding entry in `shutdown_senders`.
            let shutdown_tx = shutdown_senders.remove(&task_name).unwrap();

            log::trace!("Shutting down: {}", task_name);
            shutdown_tx.send(()).expect("error sending shutdown signal");
        }

        // Wait for all tasks to shutdown down in a certain order and maximum amount of time.
        if let Err(e) = time::timeout(SHUTDOWN_TIMEOUT, async {
            while let Some((task_name, _)) = shutdown_order.pop() {
                // Panic: unwrapping is fine, because we are in control of the data.
                let task_handle = shutdown_handles.remove(&task_name).unwrap();

                match task_handle.await {
                    Ok(_) => {
                        log::trace!("`{}` stopped.", task_name);
                    }
                    Err(e) => {
                        log::error!("Error shutting down `{task_name}`. Cause: {e}");
                    }
                }
            }
        })
        .await
        {
            log::warn!("Not all spawned tasks were shut down in time: {}.", e);
        }

        Ok(())
    }
}
