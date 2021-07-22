// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::grpc::{plugin_client::PluginClient, DummyEvent, SillyEvent};

use tokio::{
    select,
    sync::{mpsc::UnboundedReceiver, oneshot::Receiver},
};

use tokio_stream::wrappers::UnboundedReceiverStream;

use tonic::transport::Channel;

pub(crate) struct PluginStreamer<T> {
    stream: UnboundedReceiverStream<T>,
    shutdown: Receiver<()>,
    client: PluginClient<Channel>,
}

impl<T> PluginStreamer<T> {
    pub(crate) fn new(rx: UnboundedReceiver<T>, shutdown: Receiver<()>, client: PluginClient<Channel>) -> Self {
        Self {
            stream: UnboundedReceiverStream::new(rx),
            shutdown,
            client,
        }
    }
}

macro_rules! impl_streamer {
    ($ty:ty, $method:ident) => {
        impl PluginStreamer<$ty> {
            pub(crate) async fn run(mut self) {
                select! {
                    _ = &mut self.shutdown => (),
                    _ = self.client.$method(self.stream) => (),
                }
            }
        }
    };
}

impl_streamer!(DummyEvent, process_dummy_event);
impl_streamer!(SillyEvent, process_silly_event);
