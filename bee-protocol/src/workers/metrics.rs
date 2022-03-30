// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{convert::Infallible, net::SocketAddr, time::Duration};

use async_trait::async_trait;
use bee_ledger::workers::event::{MilestoneConfirmed, PrunedIndex, SnapshottedIndex};
use bee_metrics::{metrics::MemoryUsage, Registry};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::StreamExt;
use log::info;
use serde::Deserialize;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use crate::types::metrics::NodeMetrics;

const METRICS_INTERVAL: Duration = Duration::from_secs(60);
const MEMORY_METRIC_UPDATE_INTERVAL: Duration = Duration::from_secs(5);

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:3030";

#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct MetricsConfigBuilder {
    #[serde(skip)]
    pid: Option<u32>,
    #[serde(alias = "bindAddress")]
    bind_address: Option<SocketAddr>,
}

impl MetricsConfigBuilder {
    pub fn with_pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    pub fn finish(self) -> MetricsConfig {
        MetricsConfig {
            pid: self.pid.unwrap(),
            bind_address: self
                .bind_address
                .unwrap_or_else(|| DEFAULT_BIND_ADDRESS.parse().unwrap()),
        }
    }
}

#[derive(Clone)]
pub struct MetricsConfig {
    pid: u32,
    bind_address: SocketAddr,
}

pub struct MetricsWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MetricsWorker {
    type Config = MetricsConfig;
    type Error = Infallible;

    #[cfg_attr(feature = "trace", trace_tools::observe)]
    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let registry = Registry::default();

        let rss = MemoryUsage::new(config.pid);

        registry.register(
            "bee_memory_usage",
            "Memory currently allocated by the node",
            rss.clone(),
        );

        node.register_resource(NodeMetrics::new());

        let metrics = node.resource::<NodeMetrics>();
        node.bus().add_listener::<Self, MilestoneConfirmed, _>(move |event| {
            metrics.referenced_messages_inc(event.referenced_messages as u64);
            metrics.excluded_no_transaction_messages_inc(event.excluded_no_transaction_messages.len() as u64);
            metrics.excluded_conflicting_messages_inc(event.excluded_conflicting_messages.len() as u64);
            metrics.included_messages_inc(event.included_messages.len() as u64);
            metrics.created_outputs_inc(event.created_outputs as u64);
            metrics.consumed_outputs_inc(event.consumed_outputs as u64);
            metrics.receipts_inc(event.receipt as u64);
        });

        let metrics = node.resource::<NodeMetrics>();
        node.bus().add_listener::<Self, SnapshottedIndex, _>(move |_| {
            metrics.snapshots_inc(1);
        });

        let metrics = node.resource::<NodeMetrics>();
        node.bus().add_listener::<Self, PrunedIndex, _>(move |_| {
            metrics.prunings_inc(1);
        });

        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(METRICS_INTERVAL)));

            while ticker.next().await.is_some() {
                info!("{:?}", *metrics);
            }

            info!("Stopped.");
        });

        node.spawn::<Self, _, _>(|shutdown| async move {
            let mut ticker =
                ShutdownStream::new(shutdown, IntervalStream::new(interval(MEMORY_METRIC_UPDATE_INTERVAL)));

            while ticker.next().await.is_some() {
                rss.update().await;
            }
        });

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running prometheus client.");

            let serve = bee_metrics::serve_metrics(config.bind_address, registry);

            tokio::select! {
                _ = shutdown => (),
                _ = serve => (),
            };

            info!("Stopped prometheus client.");
        });

        Ok(Self {})
    }
}
