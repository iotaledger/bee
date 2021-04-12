// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use std::path::{Path, PathBuf};

const DEFAULT_FULL_PATH: &str = "./snapshots/mainnet/full_snapshot.bin";
const DEFAULT_DELTA_PATH: &str = "./snapshots/mainnet/delta_snapshot.bin";
const DEFAULT_DOWNLOAD_URLS: Vec<String> = Vec::new();
const DEFAULT_DEPTH: u32 = 50;
const DEFAULT_INTERVAL_SYNCED: u32 = 50;
const DEFAULT_INTERVAL_UNSYNCED: u32 = 1000;

#[derive(Default, Deserialize)]
pub struct SnapshotConfigBuilder {
    full_path: Option<String>,
    delta_path: Option<String>,
    download_urls: Option<Vec<String>>,
    depth: Option<u32>,
    interval_synced: Option<u32>,
    interval_unsynced: Option<u32>,
}

impl SnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn full_path(mut self, full_path: String) -> Self {
        self.full_path.replace(full_path);
        self
    }

    pub fn delta_path(mut self, delta_path: String) -> Self {
        self.delta_path.replace(delta_path);
        self
    }

    pub fn download_urls(mut self, download_urls: Vec<String>) -> Self {
        self.download_urls.replace(download_urls);
        self
    }

    pub fn depth(mut self, depth: u32) -> Self {
        self.depth.replace(depth);
        self
    }

    pub fn interval_synced(mut self, interval_synced: u32) -> Self {
        self.interval_synced.replace(interval_synced);
        self
    }

    pub fn interval_unsynced(mut self, interval_unsynced: u32) -> Self {
        self.interval_unsynced.replace(interval_unsynced);
        self
    }

    pub fn finish(self) -> SnapshotConfig {
        SnapshotConfig {
            full_path: PathBuf::from(self.full_path.unwrap_or_else(|| DEFAULT_FULL_PATH.to_string())),
            delta_path: PathBuf::from(self.delta_path.unwrap_or_else(|| DEFAULT_DELTA_PATH.to_string())),
            download_urls: self.download_urls.unwrap_or(DEFAULT_DOWNLOAD_URLS),
            depth: self.depth.unwrap_or(DEFAULT_DEPTH),
            interval_synced: self.interval_synced.unwrap_or(DEFAULT_INTERVAL_SYNCED),
            interval_unsynced: self.interval_unsynced.unwrap_or(DEFAULT_INTERVAL_UNSYNCED),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotConfig {
    full_path: PathBuf,
    delta_path: PathBuf,
    download_urls: Vec<String>,
    depth: u32,
    interval_synced: u32,
    interval_unsynced: u32,
}

impl SnapshotConfig {
    pub fn build() -> SnapshotConfigBuilder {
        SnapshotConfigBuilder::new()
    }

    pub fn full_path(&self) -> &Path {
        self.full_path.as_path()
    }

    pub fn delta_path(&self) -> &Path {
        self.delta_path.as_path()
    }

    pub fn download_urls(&self) -> &Vec<String> {
        &self.download_urls
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn interval_synced(&self) -> u32 {
        self.interval_synced
    }

    pub fn interval_unsynced(&self) -> u32 {
        self.interval_unsynced
    }
}
