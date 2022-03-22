// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible, time::Duration};

use async_trait::async_trait;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use crate::{
    types::metrics::NodeMetrics,
    workers::{event::MpsMetricsUpdated, MetricsWorker},
};

const MPS_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Default)]
pub(crate) struct MpsWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MpsWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MetricsWorker>()].leak()
    }

    #[cfg_attr(feature = "trace", trace_tools::observe)]
    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let bus = node.bus();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(MPS_INTERVAL)));

            let mut total_incoming = 0u64;
            let mut total_new = 0u64;
            let mut total_known = 0u64;
            let mut total_invalid = 0u64;
            let mut total_outgoing = 0u64;

            while ticker.next().await.is_some() {
                let incoming = metrics.messages_received();
                let new = metrics.new_messages();
                let known = metrics.known_messages();
                let invalid = metrics.invalid_messages();
                let outgoing = metrics.messages_sent();

                bus.dispatch(MpsMetricsUpdated {
                    incoming: incoming - total_incoming,
                    new: new - total_new,
                    known: known - total_known,
                    invalid: invalid - total_invalid,
                    outgoing: outgoing - total_outgoing,
                });

                total_incoming = incoming;
                total_new = new;
                total_known = known;
                total_invalid = invalid;
                total_outgoing = outgoing;
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
