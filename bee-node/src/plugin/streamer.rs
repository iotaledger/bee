// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use tokio::{
    select,
    sync::{mpsc::UnboundedReceiver, oneshot::Receiver},
};

pub(crate) struct PluginStreamer<T> {
    rx: UnboundedReceiver<T>,
    shutdown: Receiver<()>,
}

impl<T> PluginStreamer<T> {
    pub(crate) fn new(rx: UnboundedReceiver<T>, shutdown: Receiver<()>) -> Self {
        Self { rx, shutdown }
    }

    pub(crate) async fn run(&mut self) {
        loop {
            select! {
                _ = &mut self.shutdown => break,
                event = self.rx.recv() => match event {
                    Some(_event) => println!("lol event"),
                    None => break,
                }
            }
        }
    }
}
