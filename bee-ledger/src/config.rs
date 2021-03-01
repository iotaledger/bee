// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_ENABLED: bool = false;
const DEFAULT_API: &str = "http://localhost:14266";
const DEFAULT_TIMEOUT: u64 = 5;
const DEFAULT_COORDINATOR_ADDRESS: &str =
    "JFQ999DVN9CBBQX9DSAIQRAFRALIHJMYOXAQSTCJLGA9DLOKIWHJIFQKMCQ9QHWW9RXQMDBVUIQNIY9GZ";
const DEFAULT_COORDINATOR_DEPTH: usize = 18;

#[derive(Default, Deserialize)]
pub struct LedgerReceiptConfigBuilder {
    enabled: Option<bool>,
    api: Option<String>,
    timeout: Option<u64>,
    coordinator_address: Option<String>,
    coordinator_depth: Option<usize>,
}

impl LedgerReceiptConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> LedgerReceiptConfig {
        LedgerReceiptConfig {
            enabled: self.enabled.unwrap_or(DEFAULT_ENABLED),
            api: self.api.unwrap_or_else(|| DEFAULT_API.to_owned()),
            timeout: self.timeout.unwrap_or(DEFAULT_TIMEOUT),
            coordinator_address: self
                .coordinator_address
                .unwrap_or_else(|| DEFAULT_COORDINATOR_ADDRESS.to_owned()),
            coordinator_depth: self.coordinator_depth.unwrap_or(DEFAULT_COORDINATOR_DEPTH),
        }
    }
}

#[derive(Clone)]
pub struct LedgerReceiptConfig {
    enabled: bool,
    api: String,
    timeout: u64,
    coordinator_address: String,
    coordinator_depth: usize,
}

impl LedgerReceiptConfig {
    pub fn build() -> LedgerReceiptConfigBuilder {
        LedgerReceiptConfigBuilder::new()
    }
}

#[derive(Default, Deserialize)]
pub struct LedgerConfigBuilder {
    receipt: Option<LedgerReceiptConfigBuilder>,
}

impl LedgerConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> LedgerConfig {
        LedgerConfig {
            receipt: self.receipt.unwrap_or_default().finish(),
        }
    }
}

#[derive(Clone)]
pub struct LedgerConfig {
    receipt: LedgerReceiptConfig,
}

impl LedgerConfig {
    pub fn build() -> LedgerConfigBuilder {
        LedgerConfigBuilder::new()
    }
}
