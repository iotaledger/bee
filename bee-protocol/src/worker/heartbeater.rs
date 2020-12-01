// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{protocol::Protocol, tangle::MsTangle, worker::TangleWorker};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_network::Network;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::time::interval;

use std::{any::TypeId, convert::Infallible, time::Duration};

const _HEARTBEAT_SEND_INTERVAL_SEC: u64 = 30;
const _HEARTBEAT_RECEIVE_INTERVAL_SEC: u64 = 100;
const CHECK_HEARTBEATS_INTERVAL_SEC: u64 = 5;

#[derive(Default)]
pub(crate) struct HeartbeaterWorker {}

#[async_trait]
impl<N: Node> Worker<N> for HeartbeaterWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let network = node.resource::<Network>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker =
                ShutdownStream::new(shutdown, interval(Duration::from_secs(CHECK_HEARTBEATS_INTERVAL_SEC)));

            while ticker.next().await.is_some() {
                // TODO real impl
                Protocol::broadcast_heartbeat(
                    &network,
                    tangle.get_latest_solid_milestone_index(),
                    tangle.get_pruning_index(),
                    tangle.get_latest_milestone_index(),
                );
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
