// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::pruning::{PruningConfig, PruningConfigBuilder};

use serde::Deserialize;

const DEFAULT_PATH: &str = "./snapshots/mainnet/export.bin";
const DEFAULT_DOWNLOAD_URLS: Vec<String> = Vec::new();
const DEFAULT_DEPTH: u32 = 50;
const DEFAULT_INTERVAL_SYNCED: u32 = 50;
const DEFAULT_INTERVAL_UNSYNCED: u32 = 1000;

#[derive(Default, Deserialize)]
pub struct SnapshotConfigBuilder {
    path: Option<String>,
    download_urls: Option<Vec<String>>,
    depth: Option<u32>,
    interval_synced: Option<u32>,
    interval_unsynced: Option<u32>,
    pruning: Option<PruningConfigBuilder>,
}

impl SnapshotConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(mut self, path: String) -> Self {
        self.path.replace(path);
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
            path: self.path.unwrap_or_else(|| DEFAULT_PATH.to_string()),
            download_urls: self.download_urls.unwrap_or(DEFAULT_DOWNLOAD_URLS),
            depth: self.depth.unwrap_or(DEFAULT_DEPTH),
            interval_synced: self.interval_synced.unwrap_or(DEFAULT_INTERVAL_SYNCED),
            interval_unsynced: self.interval_unsynced.unwrap_or(DEFAULT_INTERVAL_UNSYNCED),
            pruning: self.pruning.unwrap_or_default().finish(),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotConfig {
    path: String,
    download_urls: Vec<String>,
    depth: u32,
    interval_synced: u32,
    interval_unsynced: u32,
    pruning: PruningConfig,
}

impl SnapshotConfig {
    pub fn build() -> SnapshotConfigBuilder {
        SnapshotConfigBuilder::new()
    }

    pub fn path(&self) -> &String {
        &self.path
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

    pub fn pruning(&self) -> &PruningConfig {
        &self.pruning
    }
}
