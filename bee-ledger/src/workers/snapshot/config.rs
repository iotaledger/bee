// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing snapshot configuration.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use url::Url;

const DEFAULT_ENABLED: bool = false;
const DEFAULT_DEPTH: u32 = 50;
const DEFAULT_INTERVAL_SYNCED: u32 = 50;
const DEFAULT_INTERVAL_UNSYNCED: u32 = 1000;
const DEFAULT_DOWNLOAD_URLS: Vec<DownloadUrls> = Vec::new();
const DEFAULT_FULL_PATH: &str = "./snapshots/mainnet/latest-full_snapshot.bin";

/// Contains URLs to download the full and delta snapshot files.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct DownloadUrls {
    full: Url,
    delta: Url,
}

impl DownloadUrls {
    /// Returns the download URL for the full snapshot.
    pub fn full(&self) -> &str {
        self.full.as_str()
    }

    /// Returns the download URL for the delta snapshot.
    pub fn delta(&self) -> &str {
        self.delta.as_str()
    }
}

/// Builder for a [`SnapshottingConfig`] which is part of the [`SnapshotConfig`].
#[derive(Default, Debug, Deserialize, PartialEq)]
#[must_use]
pub struct SnapshottingConfigBuilder {
    enabled: Option<bool>,
    depth: Option<u32>,
    #[serde(alias = "intervalSynced")]
    interval_synced: Option<u32>,
    #[serde(alias = "intervalUnsynced")]
    interval_unsynced: Option<u32>,
}

impl SnapshottingConfigBuilder {
    /// Sets whether snapshotting is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled.replace(enabled);
        self
    }

    /// Sets the snapshotting depth.
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth.replace(depth);
        self
    }

    /// Sets the snapshotting interval for a synced node.
    pub fn interval_synced(mut self, interval_synced: u32) -> Self {
        self.interval_synced.replace(interval_synced);
        self
    }

    /// Sets the snapshotting interval for an unsynced node.
    pub fn interval_unsynced(mut self, interval_unsynced: u32) -> Self {
        self.interval_unsynced.replace(interval_unsynced);
        self
    }

    /// Produces a [`SnapshottingConfig`] from this builder.
    #[must_use]
    pub fn finish(self) -> SnapshottingConfig {
        SnapshottingConfig {
            enabled: self.enabled.unwrap_or(DEFAULT_ENABLED),
            depth: self.depth.unwrap_or(DEFAULT_DEPTH),
            interval_synced: self.interval_synced.unwrap_or(DEFAULT_INTERVAL_SYNCED),
            interval_unsynced: self.interval_unsynced.unwrap_or(DEFAULT_INTERVAL_UNSYNCED),
        }
    }
}

/// Builder for a [`SnapshotConfig`] that can also be deserialized from some source.
#[derive(Default, Debug, Deserialize, PartialEq)]
#[must_use]
pub struct SnapshotConfigBuilder {
    #[serde(alias = "fullPath")]
    full_path: Option<PathBuf>,
    #[serde(alias = "deltaPath")]
    delta_path: Option<PathBuf>,
    #[serde(alias = "downloadUrls")]
    download_urls: Option<Vec<DownloadUrls>>,
    #[serde(alias = "create")]
    snapshotting: Option<SnapshottingConfigBuilder>,
}

impl SnapshotConfigBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the path of full snapshots.
    pub fn full_path(mut self, full_path: PathBuf) -> Self {
        self.full_path.replace(full_path);
        self
    }

    /// Sets the path of delta snapshots.
    pub fn delta_path(mut self, delta_path: PathBuf) -> Self {
        self.delta_path.replace(delta_path);
        self
    }

    /// Sets the URLs for downloading remotely produced snapshots.
    pub fn download_urls(mut self, download_urls: Vec<DownloadUrls>) -> Self {
        self.download_urls.replace(download_urls);
        self
    }

    /// Sets the `[SnapshottingConfigBuilder`].
    pub fn snapshotting(mut self, builder: SnapshottingConfigBuilder) -> Self {
        self.snapshotting.replace(builder);
        self
    }

    /// Produces a [`SnapshotConfig`] from this builder.
    #[must_use]
    pub fn finish(self) -> SnapshotConfig {
        SnapshotConfig {
            full_path: self
                .full_path
                .unwrap_or_else(|| PathBuf::from(DEFAULT_FULL_PATH.to_string())),
            delta_path: self.delta_path,
            download_urls: self.download_urls.unwrap_or(DEFAULT_DOWNLOAD_URLS),
            snapshotting: self.snapshotting.unwrap_or_default().finish(),
        }
    }
}

/// The configuration of downloading and creating snapshots.
#[derive(Clone, Debug)]
pub struct SnapshotConfig {
    full_path: PathBuf,
    delta_path: Option<PathBuf>,
    download_urls: Vec<DownloadUrls>,
    snapshotting: SnapshottingConfig,
}

impl SnapshotConfig {
    /// Creates a new `SnapshotConfigBuilder`.
    pub fn build() -> SnapshotConfigBuilder {
        SnapshotConfigBuilder::new()
    }

    /// Returns the full path of the `SnapshotConfig`.
    pub fn full_path(&self) -> &Path {
        self.full_path.as_path()
    }

    /// Returns the delta path of the `SnapshotConfig`.
    pub fn delta_path(&self) -> Option<&Path> {
        self.delta_path.as_deref()
    }

    /// Returns the download URLs of the `SnapshotConfig`.
    pub fn download_urls(&self) -> &[DownloadUrls] {
        &self.download_urls
    }

    /// Returns the [`SnapshottingConfig`].
    pub fn snapshotting(&self) -> &SnapshottingConfig {
        &self.snapshotting
    }
}

/// The configuration for creating snapshots.
#[derive(Clone, Debug)]
pub struct SnapshottingConfig {
    enabled: bool,
    depth: u32,
    interval_synced: u32,
    interval_unsynced: u32,
}

impl SnapshottingConfig {
    /// Returns whether snapshotting is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the snapshot depth.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Returns the snapshot interval for a synced node.
    pub fn interval_synced(&self) -> u32 {
        self.interval_synced
    }

    /// Returns the snapshot interval for an unsynced node.
    pub fn interval_unsynced(&self) -> u32 {
        self.interval_unsynced
    }
}
