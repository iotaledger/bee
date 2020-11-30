// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with the graceful shutdown of asynchronous workers.

use crate::{shutdown::Error as ShutdownError, worker::Error as WorkerError};

use futures::channel::oneshot;
use log::error;
// use tokio::task::JoinError;
// use tokio::runtime::task::JoinError;
use tokio::task::JoinError;

use std::future::Future;

/// A type alias for the sending side of the shutdown signal.
pub type ShutdownNotifier = oneshot::Sender<()>;

/// A type alias for the receiving side of the shutdown signal.
pub type ShutdownListener = oneshot::Receiver<()>;

/// A type alias for the termination `Future` of an asynchronous worker.
pub type WorkerShutdown = Box<dyn Future<Output = Result<Result<(), WorkerError>, JoinError>> + Unpin + Send>;

/// A type alias for a closure, that is executed during the final step of the shutdown procedure.
pub type Action = Box<dyn FnOnce() + Send>;

/// Handles the graceful shutdown of asynchronous workers.
#[derive(Default)]
pub struct Shutdown {
    notifiers: Vec<ShutdownNotifier>,
    worker_shutdowns: Vec<WorkerShutdown>,
    actions: Vec<Action>,
}

impl Shutdown {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an asynchronous worker, and a corresponding shutdown notifier.
    pub fn add_worker_shutdown(
        &mut self,
        notifier: ShutdownNotifier,
        worker: impl Future<Output = Result<Result<(), WorkerError>, JoinError>> + Unpin + Send + 'static,
    ) {
        self.notifiers.push(notifier);
        self.worker_shutdowns.push(Box::new(worker));
    }

    /// Adds teardown logic that is executed during shutdown.
    pub fn add_action(&mut self, action: impl FnOnce() + Send + 'static) {
        self.actions.push(Box::new(action));
    }

    /// Executes the shutdown.
    pub async fn execute(mut self) -> Result<(), ShutdownError> {
        while let Some(notifier) = self.notifiers.pop() {
            // NOTE: in case of an error the `Err` variant simply contains our shutdown signal `()` that we tried to
            // send.
            notifier
                .send(())
                .map_err(|_| ShutdownError::SendingShutdownSignalFailed)?
        }

        while let Some(worker_shutdown) = self.worker_shutdowns.pop() {
            if let Err(e) = worker_shutdown.await {
                error!("Awaiting worker failed: {:?}.", e);
            }
        }

        while let Some(action) = self.actions.pop() {
            action();
        }

        Ok(())
    }
}
