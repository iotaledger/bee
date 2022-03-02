// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Version checker plugin for the Bee node.

#![warn(missing_docs)]

mod release_info;

use crate::release_info::{ReleaseInfo, ReleaseInfoBuilder};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{convert::Infallible, time::Duration};

const CHECK_INTERVAL_SEC: u64 = 3600;

type VersionCheckerConfig = String;

/// Version checker plugin.
#[derive(Default)]
pub struct VersionCheckerPlugin;

#[async_trait]
impl<N: Node> Worker<N> for VersionCheckerPlugin {
    type Config = VersionCheckerConfig;
    type Error = Infallible;

    async fn start(node: &mut N, current_version: Self::Config) -> Result<Self, Self::Error> {
        node.spawn::<Self, _, _>(|shutdown| async move {
            log::info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(CHECK_INTERVAL_SEC))),
            );

            while ticker.next().await.is_some() {
                show_version_status(&current_version).await;
            }

            log::info!("Stopped.");
        });

        Ok(Self::default())
    }
}

/// Alerts the user (through a `WARN` level log event) if there is a newer release version of Bee available,
/// and provides a URL to the release's summary webpage.
async fn show_version_status(current_version: &str) {
    if let Some(release_info) = get_release_info().await {
        match (semver::Version::parse(current_version), latest(release_info)) {
            (Err(e), _) => println!("Error parsing current Bee version ({}): {}", current_version, e),
            (Ok(current), Some(latest)) => {
                if latest.version > current {
                    log::warn!(
                        "Found a more recent release version ({}), available at {}",
                        latest.version,
                        latest.html_url,
                    );
                } else {
                    log::info!("On the latest release version ({})", current_version);
                }
            }
            _ => log::warn!("Could not identify the latest release version"),
        }
    }
}

/// Returns a list of [`ReleaseInfoBuilder`]s describing all Github releases, if the API request and
/// deserialization was successful. Otherwise, returns `None`.
async fn get_release_info() -> Option<Vec<ReleaseInfoBuilder>> {
    let client = reqwest::Client::builder().user_agent("bee").build().unwrap();

    match client
        .get("https://api.github.com/repos/iotaledger/bee/releases")
        .send()
        .await
        .and_then(|resp| resp.error_for_status())
    {
        Ok(resp) => match resp.json::<Vec<ReleaseInfoBuilder>>().await {
            Ok(releases) => Some(releases),
            Err(e) => {
                // Error deserializing response.
                log::error!("{}", e);
                None
            }
        },
        Err(e) => {
            // Error occured during API request.
            log::error!("{}", e);
            None
        }
    }
}

/// Returns the latest full release (i.e. not a pre-release version) given a vector of [`ReleaseInfoBuilder`]s.
fn latest(release_info: Vec<ReleaseInfoBuilder>) -> Option<ReleaseInfo> {
    let mut releases = release_info
        .into_iter()
        .filter_map(|info| info.build())
        .collect::<Vec<_>>();

    releases.sort();
    releases.pop()
}
