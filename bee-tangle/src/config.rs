// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::pruning::{PruningConfig, PruningConfigBuilder};

use serde::Deserialize;

const DEFAULT_BELOW_MAX_DEPTH: u32 = 15;

#[derive(Default, Deserialize)]
pub struct TangleConfigBuilder {
    below_max_depth: Option<u32>,
    pruning: Option<PruningConfigBuilder>,
}

impl TangleConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> TangleConfig {
        TangleConfig {
            below_max_depth: self.below_max_depth.unwrap_or(DEFAULT_BELOW_MAX_DEPTH),
            pruning: self.pruning.unwrap_or_default().finish(),
        }
    }
}

#[derive(Clone)]
pub struct TangleConfig {
    below_max_depth: u32,
    pruning: PruningConfig,
}

impl TangleConfig {
    pub fn build() -> TangleConfigBuilder {
        TangleConfigBuilder::new()
    }

    pub fn below_max_depth(&self) -> u32 {
        self.below_max_depth
    }

    pub fn pruning(&self) -> &PruningConfig {
        &self.pruning
    }
}
