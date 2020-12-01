// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;

use std::{convert::Infallible, time::Duration};

const CHECK_INTERVAL_SEC: u64 = 3600;

#[derive(Default)]
pub(crate) struct VersionCheckerWorker {}

#[async_trait]
impl<N: Node> Worker<N> for VersionCheckerWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(shutdown, interval(Duration::from_secs(CHECK_INTERVAL_SEC)));

            while ticker.next().await.is_some() {
                // TODO
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
