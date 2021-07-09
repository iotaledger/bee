// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::constants::{BEE_GIT_API_LATEST, BEE_VERSION};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::{error, info, warn};
use reqwest::Client;
use serde_json::Value;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::time::Duration;

const CHECK_INTERVAL_SEC: u64 = 3600;

#[derive(Default)]
pub struct VersionChecker {}

#[async_trait]
impl<N: Node> Worker<N> for VersionChecker {
    type Config = ();
    type Error = reqwest::Error;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {

        let client = reqwest::Client::builder()
                .user_agent("curl")
                .build()?;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(CHECK_INTERVAL_SEC))),
            );

            while ticker.next().await.is_some() {
                match is_latest(&client).await {
                    Ok(false) => warn!("A new version has been released. Please update the node at https://github.com/iotaledger/bee/releases"),
                    Err(e) => error!("error while checking for new update. {:?}", e),
                    _ => (),
                }
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}

async fn is_latest(client: &Client) -> Result<bool, reqwest::Error> {
    let res = client.get(BEE_GIT_API_LATEST).send().await?.error_for_status()?;

    match res.text().await {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => match value.get("tag_name") {
                Some(tag_name) => return Ok(tag_name == format!("v{}", BEE_VERSION).as_str()),
                None => error!("no version field found."),
            },
            Err(e) => error!("error while getting update information. {:?}", e),
        },
        Err(e) => error!("error while getting update information. {:?}", e),
    }

    Ok(true)
}
