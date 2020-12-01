// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{tangle::MsTangle, worker::TangleWorker};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;

use std::{any::TypeId, convert::Infallible, time::Duration};

const TIP_POOL_CLEANER_INTERVAL_SEC: u64 = 1;

#[derive(Default)]
pub(crate) struct TipPoolCleanerWorker {}

#[async_trait]
impl<N: Node> Worker<N> for TipPoolCleanerWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker =
                ShutdownStream::new(shutdown, interval(Duration::from_secs(TIP_POOL_CLEANER_INTERVAL_SEC)));

            while ticker.next().await.is_some() {
                tangle.reduce_tips().await
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
