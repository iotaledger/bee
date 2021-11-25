// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::workers::{storage::StorageBackend, MessageRequesterWorker, RequestedMessages};

use backstage::core::{Actor, ActorError, ActorResult, IntervalChannel, Rt, StreamExt, SupHandle};
use bee_ledger::workers::consensus::ConsensusWorker;
use bee_runtime::{node::Node, resource::ResourceHandle, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};

use async_trait::async_trait;
use log::info;

use std::{any::TypeId, convert::Infallible, marker::PhantomData};

pub struct StatusActor<B: StorageBackend> {
    _marker: PhantomData<(B,)>,
}

impl<B: StorageBackend> Default for StatusActor<B> {
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

#[async_trait]
impl<S, B: StorageBackend> Actor<S> for StatusActor<B>
where
    S: SupHandle<Self>,
{
    type Data = (ResourceHandle<Tangle<B>>, ResourceHandle<RequestedMessages>);
    // FIXME: interval is passed through configuration and it is not a constant.
    type Channel = IntervalChannel<1000>;

    async fn init(&mut self, rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        let parent_id = rt
            .parent_id()
            .ok_or_else(|| ActorError::aborted_msg("actor has no parent"))?;

        let tangle = rt
            .lookup::<ResourceHandle<Tangle<B>>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("resource is not available"))?;

        let requested_messages = rt
            .lookup::<ResourceHandle<RequestedMessages>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("resource is not available"))?;

        Ok((tangle, requested_messages))
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, (tangle, requested_messages): Self::Data) -> ActorResult<()> {
        info!("Running.");

        while rt.inbox_mut().next().await.is_some() {
            let snapshot_index = *tangle.get_snapshot_index();
            let confirmed_milestone_index = *tangle.get_confirmed_milestone_index();
            let solid_milestone_index = *tangle.get_solid_milestone_index();
            let latest_milestone_index = *tangle.get_latest_milestone_index();

            let status = if confirmed_milestone_index == latest_milestone_index {
                format!("Synchronized and confirmed at {}", latest_milestone_index)
            } else {
                let confirmed_progress = ((confirmed_milestone_index - snapshot_index) as f64 * 100.0
                    / (latest_milestone_index - snapshot_index) as f64) as u8;
                let solid_progress = ((solid_milestone_index - snapshot_index) as f32 * 100.0
                    / (latest_milestone_index - snapshot_index) as f32) as u8;
                format!(
                    "Synchronizing from {} to {}: confirmed {} ({}%) and solid {} ({}%) - Requested {}",
                    snapshot_index,
                    latest_milestone_index,
                    confirmed_milestone_index,
                    confirmed_progress,
                    solid_milestone_index,
                    solid_progress,
                    requested_messages.len().await,
                )
            };

            info!("{} - Tips {}.", status, tangle.non_lazy_tips_num().await);
        }

        info!("Stopped.");

        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct StatusWorker;

#[async_trait]
impl<N: Node> Worker<N> for StatusWorker
where
    N::Backend: StorageBackend,
{
    type Config = u64;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MessageRequesterWorker>(),
            TypeId::of::<ConsensusWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async {
            shutdown.await.unwrap();
        });

        Ok(Self::default())
    }
}
