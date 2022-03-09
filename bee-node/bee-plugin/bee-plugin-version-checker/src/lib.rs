// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Version checker plugin for the Bee node.

#![warn(missing_docs)]

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{convert::Infallible, time::Duration};

const CHECK_INTERVAL_SEC: u64 = 3600;

/// Version checker plugin.
#[derive(Default)]
pub struct VersionCheckerPlugin;

#[async_trait]
impl<N: Node> Worker<N> for VersionCheckerPlugin {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(CHECK_INTERVAL_SEC))),
            );

            while ticker.next().await.is_some() {
                // TODO
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
