// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::{future::FutureExt, stream::StreamExt};
use log::{debug, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    {storage::StorageBackend, MetricsWorker},
};

pub(crate) struct TaggedDataPayloadWorkerEvent {}

pub(crate) struct TaggedDataPayloadWorker {
    pub(crate) tx: mpsc::UnboundedSender<TaggedDataPayloadWorkerEvent>,
}

#[async_trait]
impl<N> Worker<N> for TaggedDataPayloadWorker
where
    N: Node,
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let metrics = node.resource::<NodeMetrics>();
        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(TaggedDataPayloadWorkerEvent {}) = receiver.next().await {
                metrics.tagged_data_payload_inc(1);
            }

            // Before the worker completely stops, the receiver needs to be drained for tagged data payloads to be
            // analysed. Otherwise, information would be lost and not easily recoverable.

            let (_, mut receiver) = receiver.split();
            let mut count: usize = 0;

            while let Some(Some(TaggedDataPayloadWorkerEvent {})) = receiver.next().now_or_never() {
                metrics.tagged_data_payload_inc(1);
                count += 1;
            }

            debug!("Drained {} events.", count);

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
