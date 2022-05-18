// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{convert::Infallible, net::SocketAddr, time::Duration};

use async_trait::async_trait;
use bee_metrics::{metrics::process::ProcessMetrics, Registry};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::StreamExt;
use log::{debug, info};
use serde::Deserialize;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

const PROCESS_METRICS_UPDATE_INTERVAL: Duration = Duration::from_secs(5);
const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:3030";

#[derive(Default, Deserialize, PartialEq)]
#[must_use]
pub struct MetricsConfigBuilder {
    #[serde(alias = "bindAddress")]
    bind_address: Option<SocketAddr>,
}

impl MetricsConfigBuilder {
    pub fn finish(self, pid: u32) -> MetricsConfig {
        MetricsConfig {
            pid,
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

pub struct MetricsRegistryWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MetricsRegistryWorker {
    type Config = MetricsConfig;
    type Error = Infallible;

    #[cfg_attr(feature = "trace", trace_tools::observe)]
    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let registry = node.resource::<Registry>();

        let process_metrics = ProcessMetrics::new(config.pid);
        let (mem_metric, cpu_metric) = process_metrics.metrics();

        registry.register("bee_memory_usage", "Memory currently allocated by the node", mem_metric);
        registry.register("bee_cpu_usage", "CPU pecentage currently used by the node", cpu_metric);

        node.spawn::<Self, _, _>(|shutdown| async move {
            let mut ticker =
                ShutdownStream::new(shutdown, IntervalStream::new(interval(PROCESS_METRICS_UPDATE_INTERVAL)));

            while ticker.next().await.is_some() {
                if let Err(e) = process_metrics.update().await {
                    debug!("Cannot update process metrics: {e}.");
                }
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
