// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{event::MpsMetricsUpdated, MetricsWorker},
};

use bee_runtime::{event::Bus, node::Node, resource::ResourceHandle, worker::Worker};

use async_trait::async_trait;
use backstage::core::{Actor, ActorError, ActorResult, IntervalChannel, Rt, SupHandle};
use futures::StreamExt;
use log::info;

use std::{any::TypeId, convert::Infallible};

// TODO: It feels like the MpsActor is being run a lot more than just once per second.
const MPS_INTERVAL: u64 = 1; // In seconds

#[derive(Default)]
pub struct MpsActor {}

#[async_trait]
impl<S> Actor<S> for MpsActor
where
    S: SupHandle<Self>,
{
    type Data = (ResourceHandle<Bus<'static>>, ResourceHandle<NodeMetrics>);
    type Channel = IntervalChannel<MPS_INTERVAL>;

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

        let mut total_incoming = 0u64;
        let mut total_new = 0u64;
        let mut total_known = 0u64;
        let mut total_invalid = 0u64;
        let mut total_outgoing = 0u64;

        while rt.inbox_mut().next().await.is_some() {
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

        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct MpsWorker {}

#[async_trait]
impl<N: Node> Worker<N> for MpsWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            shutdown.await.unwrap();

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
