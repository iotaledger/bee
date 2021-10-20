// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hasher;

use tokio::sync::oneshot;

pub(crate) type ShutdownRx = oneshot::Receiver<()>;
type ShutdownTx = oneshot::Sender<()>;

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
