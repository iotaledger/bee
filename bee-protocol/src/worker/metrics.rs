// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::ProtocolMetrics;

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::{delay_for, interval};

use std::{convert::Infallible, time::Duration};

const METRICS_INTERVAL_S: u64 = 60;

pub(crate) struct MetricsWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MetricsWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.register_resource(ProtocolMetrics::new());
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            delay_for(Duration::from_secs(METRICS_INTERVAL_S)).await;

            let mut ticker = ShutdownStream::new(shutdown, interval(Duration::from_secs(METRICS_INTERVAL_S)));

            while ticker.next().await.is_some() {
                info!("{:?}", *metrics);
            }

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
