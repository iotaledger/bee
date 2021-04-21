// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_BELOW_MAX_DEPTH: u32 = 15;

#[derive(Default, Deserialize)]
pub struct TangleConfigBuilder {
    below_max_depth: Option<u32>,
}

impl TangleConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> TangleConfig {
        TangleConfig {
            below_max_depth: self.below_max_depth.unwrap_or(DEFAULT_BELOW_MAX_DEPTH),
        }
    }
}

#[derive(Clone)]
pub struct TangleConfig {
    below_max_depth: u32,
}

impl TangleConfig {
    pub fn build() -> TangleConfigBuilder {
        TangleConfigBuilder::new()
    }

    pub fn below_max_depth(&self) -> u32 {
        self.below_max_depth
    }
}
