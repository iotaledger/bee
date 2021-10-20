// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::sync::oneshot;

pub(crate) type ShutdownRx = oneshot::Receiver<()>;

pub(crate) struct ShutdownBus {
    senders: Vec<oneshot::Sender<()>>,
}

impl ShutdownBus {
    pub fn new(n: usize) -> (Self, Vec<oneshot::Receiver<()>>) {
        let mut senders = Vec::with_capacity(n);
        let mut receivers = Vec::with_capacity(n);

        for _ in 0..n {
            let (tx, rx) = oneshot::channel::<()>();
            senders.push(tx);
            receivers.push(rx);
        }

        (Self { senders }, receivers)
    }

    pub fn trigger(self) {
        for s in self.senders {
            s.send(()).expect("error sending shutdown signal");
        }
    }
}
