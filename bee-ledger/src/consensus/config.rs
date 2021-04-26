// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

/// Builder for a `LedgerConfig`.
#[derive(Default, Deserialize)]
pub struct LedgerConfigBuilder {}

impl LedgerConfigBuilder {
    /// Creates a new `LedgerConfigBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a `LedgerConfig` from a `LedgerConfigBuilder`.
    pub fn finish(self) -> LedgerConfig {
        LedgerConfig {}
    }
}

/// Configuration of the ledger.
#[derive(Clone)]
pub struct LedgerConfig {}

impl LedgerConfig {
    /// Creates a builder for a `LedgerConfig`.
    pub fn build() -> LedgerConfigBuilder {
        LedgerConfigBuilder::new()
    }
}
