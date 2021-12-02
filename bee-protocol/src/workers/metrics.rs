// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::metrics::NodeMetrics;

use bee_ledger::workers::event::{MilestoneConfirmed, PrunedIndex, SnapshottedIndex};
use bee_runtime::{event::Bus, node::Node, resource::ResourceHandle, worker::Worker};

use async_trait::async_trait;
use backstage::core::{Actor, ActorError, ActorResult, IntervalChannel, Rt, SupHandle};
use futures::StreamExt;
use log::info;

use std::{convert::Infallible, time::Duration};

const METRICS_INTERVAL: u64 = Duration::from_secs(60).as_millis() as u64;

#[derive(Default)]
pub struct MetricsActor {}

#[async_trait]
impl<S> Actor<S> for MetricsActor
where
    S: SupHandle<Self>,
{
    type Data = (ResourceHandle<Bus<'static>>, ResourceHandle<NodeMetrics>);
    type Channel = IntervalChannel<METRICS_INTERVAL>;

    async fn init(&mut self, rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        // This should be the ID of the supervisor.
        let parent_id = rt
            .parent_id()
            .ok_or_else(|| ActorError::aborted_msg("gossip actor has no parent"))?;

        // The event bus should be under the supervisor's ID.
        let bus = rt
            .lookup(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("event bus is not available"))?;

        // The node metrics should be under the supervisor's ID.
        let node_metrics = rt
            .lookup(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("event bus is not available"))?;

        Ok((bus, node_metrics))
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, (bus, metrics): Self::Data) -> ActorResult<()> {
        rt.add_resource(ResourceHandle::clone(&metrics)).await;

        {
            let metrics = ResourceHandle::clone(&metrics);
            bus.add_listener::<Self, MilestoneConfirmed, _>(move |event| {
                metrics.referenced_messages_inc(event.referenced_messages as u64);
                metrics.excluded_no_transaction_messages_inc(event.excluded_no_transaction_messages.len() as u64);
                metrics.excluded_conflicting_messages_inc(event.excluded_conflicting_messages.len() as u64);
                metrics.included_messages_inc(event.included_messages.len() as u64);
                metrics.created_outputs_inc(event.created_outputs as u64);
                metrics.consumed_outputs_inc(event.consumed_outputs as u64);
                metrics.receipts_inc(event.receipt as u64);
            });
        }

        {
            let metrics = ResourceHandle::clone(&metrics);
            bus.add_listener::<Self, SnapshottedIndex, _>(move |_| {
                metrics.snapshots_inc(1);
            });
        }

        {
            let metrics = ResourceHandle::clone(&metrics);
            bus.add_listener::<Self, PrunedIndex, _>(move |_| {
                metrics.prunings_inc(1);
            });
        }

        while rt.inbox_mut().next().await.is_some() {
            info!("Metrics from backstage: {:?}", *metrics);
        }

        info!("Stopped.");

        Ok(())
    }
}

#[deprecated]
pub struct MetricsWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MetricsWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.register_resource(NodeMetrics::new());

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            shutdown.await.unwrap();

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
