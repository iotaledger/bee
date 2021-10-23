// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::delay::DelayedRepeat;

use std::{future::Future, hash::Hasher};

use tokio::sync::oneshot;

pub(crate) type ShutdownRx = oneshot::Receiver<()>;
type ShutdownTx = oneshot::Sender<()>;

#[async_trait::async_trait]
pub(crate) trait Runnable {
    const NAME: &'static str;

    type Cancel: Future + Send + Unpin + 'static;

    async fn run(self, cancel: Self::Cancel);
}

pub(crate) struct Spawner {}

impl Spawner {
    pub(crate) fn spawn_runnable<R>(runnable: R, cancel: ShutdownRx)
    where
        R: Runnable<Cancel = ShutdownRx> + 'static,
    {
        log::info!("Running `{}`.", R::NAME);

        tokio::spawn(runnable.run(cancel));
    }

    pub(crate) fn spawn<F>(fut: F, task_name: &str)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        log::info!("Spawning `{}` task.", task_name);

        tokio::spawn(fut);
    }
}

pub(crate) struct ShutdownBus<const N: usize> {
    senders: Vec<ShutdownTx>,
}

impl<const N: usize> ShutdownBus<N> {
    pub fn new() -> (Self, ShutdownBusRegistry) {
        let mut senders = Vec::with_capacity(N);
        let mut receivers = Vec::with_capacity(N);

        (0..N).for_each(|_| {
            let (tx, rx) = oneshot::channel::<()>();
            senders.push(tx);
            receivers.push(rx);
        });

        (Self { senders }, ShutdownBusRegistry(receivers))
    }

    pub fn trigger(self) {
        for s in self.senders {
            s.send(()).expect("error sending shutdown signal");
        }
    }
}

pub(crate) struct ShutdownBusRegistry(Vec<ShutdownRx>);

impl ShutdownBusRegistry {
    pub fn register(&mut self) -> ShutdownRx {
        self.0.pop().expect("too many registrees")
    }
}
