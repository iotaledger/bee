// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing snapshot configuration.

use serde::Deserialize;
use url::Url;

use std::path::{Path, PathBuf};

const DEFAULT_FULL_PATH: &str = "./snapshots/mainnet/latest-full_snapshot.bin";
const DEFAULT_DOWNLOAD_URLS: Vec<DownloadUrls> = Vec::new();
const DEFAULT_DEPTH: u32 = 50;
const DEFAULT_INTERVAL_SYNCED: u32 = 50;
const DEFAULT_INTERVAL_UNSYNCED: u32 = 1000;

/// Contains URLs to download the full and delta snapshot files.
#[derive(Clone, Deserialize)]
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

/// Builder for a `SnapshotConfig`.
#[derive(Default, Deserialize)]
pub struct SnapshotConfigBuilder {
    full_path: Option<PathBuf>,
    delta_path: Option<PathBuf>,
    download_urls: Option<Vec<DownloadUrls>>,
    depth: Option<u32>,
    interval_synced: Option<u32>,
    interval_unsynced: Option<u32>,
}

impl SnapshotConfigBuilder {
    /// Creates a new `SnapshotConfigBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the full path of the `SnapshotConfigBuilder`.
    pub fn full_path(mut self, full_path: PathBuf) -> Self {
        self.full_path.replace(full_path);
        self
    }

    /// Sets the delta path of the `SnapshotConfigBuilder`.
    pub fn delta_path(mut self, delta_path: PathBuf) -> Self {
        self.delta_path.replace(delta_path);
        self
    }

    /// Sets the download URLs of the `SnapshotConfigBuilder`.
    pub fn download_urls(mut self, download_urls: Vec<DownloadUrls>) -> Self {
        self.download_urls.replace(download_urls);
        self
    }

    /// Sets the depth of the `SnapshotConfigBuilder`.
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth.replace(depth);
        self
    }

    /// Sets the synced interval of the `SnapshotConfigBuilder`.
    pub fn interval_synced(mut self, interval_synced: u32) -> Self {
        self.interval_synced.replace(interval_synced);
        self
    }

    /// Sets the unsynced interval of the `SnapshotConfigBuilder`.
    pub fn interval_unsynced(mut self, interval_unsynced: u32) -> Self {
        self.interval_unsynced.replace(interval_unsynced);
        self
    }

    /// Finishes the `SnapshotConfigBuilder` into a `SnapshotConfig`.
    pub fn finish(self) -> SnapshotConfig {
        SnapshotConfig {
            full_path: self
                .full_path
                .unwrap_or_else(|| PathBuf::from(DEFAULT_FULL_PATH.to_string())),
            delta_path: self.delta_path,
            download_urls: self.download_urls.unwrap_or(DEFAULT_DOWNLOAD_URLS),
            depth: self.depth.unwrap_or(DEFAULT_DEPTH),
            interval_synced: self.interval_synced.unwrap_or(DEFAULT_INTERVAL_SYNCED),
            interval_unsynced: self.interval_unsynced.unwrap_or(DEFAULT_INTERVAL_UNSYNCED),
        }
    }
}

/// A snapshot configuration.
#[derive(Clone)]
pub struct SnapshotConfig {
    full_path: PathBuf,
    delta_path: Option<PathBuf>,
    download_urls: Vec<DownloadUrls>,
    depth: u32,
    interval_synced: u32,
    interval_unsynced: u32,
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

    /// Returns the depth of the `SnapshotConfig`.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Returns the synced interval of the `SnapshotConfig`.
    pub fn interval_synced(&self) -> u32 {
        self.interval_synced
    }

    /// Returns the unsynced interval of the `SnapshotConfig`.
    pub fn interval_unsynced(&self) -> u32 {
        self.interval_unsynced
    }
}
