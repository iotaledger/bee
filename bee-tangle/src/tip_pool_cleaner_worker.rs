// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{storage::StorageBackend, Tangle, TangleWorker};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{any::TypeId, convert::Infallible, time::Duration};

// In seconds
const TIP_POOL_CLEANER_INTERVAL: u64 = 1;

#[derive(Default)]
pub(crate) struct TipPoolCleanerWorker {}

#[async_trait]
impl<N: Node> Worker<N> for TipPoolCleanerWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<Tangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(TIP_POOL_CLEANER_INTERVAL))),
            );

            while ticker.next().await.is_some() {
                tangle.reduce_tips().await
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
