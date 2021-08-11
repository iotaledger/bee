// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{event::*, grpc::plugin_client::PluginClient};

use tokio::{
    select,
    sync::{mpsc::UnboundedReceiver, oneshot::Receiver},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::transport::Channel;

/// Type that streams events of the specified type to the gRPC server of a plugin.
pub(crate) struct PluginStreamer<T> {
    stream: UnboundedReceiverStream<T>,
    shutdown: Receiver<()>,
    client: PluginClient<Channel>,
}

impl<T> PluginStreamer<T> {
    pub(crate) fn new(stream: UnboundedReceiver<T>, shutdown: Receiver<()>, client: PluginClient<Channel>) -> Self {
        Self {
            stream: UnboundedReceiverStream::new(stream),
            shutdown,
            client,
        }
    }
}

macro_rules! impl_streamer {
    ($($event_ty:ty => $method_name:ident),*) => {
        $(
            impl PluginStreamer<$event_ty> {
                pub(crate) async fn run(mut self) {
                    select! {
                        _ = &mut self.shutdown => (),
                        _ = self.client.$method_name(self.stream) => (),
                    }
                }
            }
        )*
    };
}

impl_streamer! {
    MessageParsedEvent      => process_message_parsed_event,
    ParsingFailedEvent      => process_parsing_failed_event,
    MessageRejectedEvent    => process_message_rejected_event
}
