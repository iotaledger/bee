// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::pruning::{PruningConfig, PruningConfigBuilder};

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct TangleConfigBuilder {
    pruning: Option<PruningConfigBuilder>,
}

impl TangleConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> TangleConfig {
        TangleConfig {
            pruning: self.pruning.unwrap_or_default().finish(),
        }
    }
}

#[derive(Clone)]
pub struct TangleConfig {
    pruning: PruningConfig,
}

impl TangleConfig {
    pub fn build() -> TangleConfigBuilder {
        TangleConfigBuilder::new()
    }

    pub fn pruning(&self) -> &PruningConfig {
        &self.pruning
    }
}
