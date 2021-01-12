// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct TangleConfigBuilder {}

impl TangleConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> TangleConfig {
        TangleConfig {}
    }
}

#[derive(Clone)]
pub struct TangleConfig {}

impl TangleConfig {
    pub fn build() -> TangleConfigBuilder {
        TangleConfigBuilder::new()
    }
}
