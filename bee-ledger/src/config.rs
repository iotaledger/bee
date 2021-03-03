// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct LedgerConfigBuilder {}

impl LedgerConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> LedgerConfig {
        LedgerConfig {}
    }
}

#[derive(Clone)]
pub struct LedgerConfig {}

impl LedgerConfig {
    pub fn build() -> LedgerConfigBuilder {
        LedgerConfigBuilder::new()
    }
}
