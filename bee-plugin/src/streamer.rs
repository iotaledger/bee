// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::PluginError,
    grpc::{plugin_client::PluginClient, DummyEvent},
};

use tokio::{
    select,
    sync::{mpsc::UnboundedReceiver, oneshot::Receiver},
};
use tonic::transport::Channel;

pub(crate) struct PluginStreamer<T> {
    rx: UnboundedReceiver<T>,
    shutdown: Receiver<()>,
    client: PluginClient<Channel>,
}

impl<T> PluginStreamer<T> {
    pub(crate) fn new(rx: UnboundedReceiver<T>, shutdown: Receiver<()>, client: PluginClient<Channel>) -> Self {
        Self { rx, shutdown, client }
    }
}

impl PluginStreamer<DummyEvent> {
    pub(crate) async fn run(&mut self) {
        loop {
            select! {
                _ = &mut self.shutdown => break,
                event = self.rx.recv() => match event {
                    Some(event) => {
                        if let Err(err) = self.client.process_dummy_event(event).await {
                            eprintln!("{}", PluginError::from(err))
                        }
                    },
                    None => break,
                }
            }
        }
    }
}
