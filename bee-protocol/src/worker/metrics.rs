// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::ProtocolMetrics;

use bee_ledger::consensus::event::MilestoneConfirmed;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{convert::Infallible, time::Duration};

const METRICS_INTERVAL_SEC: u64 = 60;

pub struct MetricsWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MetricsWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.register_resource(ProtocolMetrics::new());
        let metrics = node.resource::<ProtocolMetrics>();

        node.bus().add_listener::<Self, MilestoneConfirmed, _>(move |event| {
            metrics.referenced_messages_inc(event.referenced_messages as u64);
            metrics.excluded_no_transaction_messages_inc(event.excluded_no_transaction_messages.len() as u64);
            metrics.excluded_conflicting_messages_inc(event.excluded_conflicting_messages.len() as u64);
            metrics.included_messages_inc(event.included_messages.len() as u64);
            metrics.created_outputs_inc(event.created_outputs as u64);
            metrics.consumed_outputs_inc(event.consumed_outputs as u64);
        });

        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(METRICS_INTERVAL_SEC))),
            );

            while ticker.next().await.is_some() {
                info!("{:?}", *metrics);
            }

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
