// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Version checker plugin for the Bee node.

#![warn(missing_docs)]

mod release_info;

use std::{convert::Infallible, time::Duration};

use async_trait::async_trait;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::StreamExt;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use self::release_info::{ReleaseInfo, ReleaseInfoBuilder};

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
                if let Some(release_info) = get_release_info().await {
                    show_version_status(&current_version, release_info).await;
                }
            }

            log::info!("Stopped.");
        });

        Ok(Self::default())
    }
}

/// Alerts the user (through a `WARN` level log event) if there is a newer release version of Bee available,
/// and provides a URL to the release's summary webpage.
async fn show_version_status(current_version: &str, release_info: Vec<ReleaseInfoBuilder>) {
    let current_version = match semver::Version::parse(current_version) {
        Err(e) => {
            log::error!("Error parsing current Bee version ({}): {}.", current_version, e);
            return;
        }
        Ok(current_version) => current_version,
    };

    match latest(release_info, !current_version.pre.is_empty()) {
        Some(latest) => {
            if latest.version > current_version {
                log::warn!(
                    "Found a more recent release version ({}), available at {}.",
                    latest.version,
                    latest.html_url,
                );
            } else {
                log::info!("On the latest release version ({}).", current_version);
            }
        }
        None => log::warn!("Could not identify the latest release version."),
    }
}

/// Returns a list of [`ReleaseInfoBuilder`]s describing all Github releases if the API request and
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
                log::error!("Error deserializing Github API reponse: {}", e);
                None
            }
        },
        Err(e) => {
            // Error occured during API request.
            log::error!("Error performing Github API request: {}", e);
            None
        }
    }
}

/// Returns the latest release given a vector of [`ReleaseInfoBuilder`]s.
///
/// If the current `bee-node` version is a pre-release, this will include later pre-release versions.
fn latest(release_info: Vec<ReleaseInfoBuilder>, pre_release: bool) -> Option<ReleaseInfo> {
    release_info
        .into_iter()
        .filter_map(|info| info.build(pre_release))
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_release() {
        let current_version = semver::Version::parse("0.2.0").unwrap();
        let release_info = vec![
            ReleaseInfoBuilder {
                html_url: "".to_string(),
                tag_name: "v0.1.0".to_string(),
            },
            ReleaseInfoBuilder {
                html_url: "".to_string(),
                tag_name: "v0.2.0".to_string(),
            },
            ReleaseInfoBuilder {
                html_url: "".to_string(),
                tag_name: "v0.3.0".to_string(),
            },
        ];

        let latest = latest(release_info, false).unwrap();

        assert!(current_version < latest.version);
        assert_eq!(latest.version.to_string(), "0.3.0".to_string());
    }

    #[test]
    fn pre_release() {
        let current_version = semver::Version::parse("0.3.0-rc4").unwrap();
        let release_info = vec![
            ReleaseInfoBuilder {
                html_url: "".to_string(),
                tag_name: "v0.2.0".to_string(),
            },
            ReleaseInfoBuilder {
                html_url: "".to_string(),
                tag_name: "v0.3.0-rc4".to_string(),
            },
            ReleaseInfoBuilder {
                html_url: "".to_string(),
                tag_name: "v0.3.0-rc5".to_string(),
            },
        ];

        let latest = latest(release_info, true).unwrap();

        assert!(current_version < latest.version);
        assert_eq!(latest.version.to_string(), "0.3.0-rc5".to_string());
    }
}
