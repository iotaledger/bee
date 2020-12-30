// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_PORT: u16 = 8081;

#[derive(Default, Deserialize)]
pub struct DashboardConfigBuilder {
    port: Option<u16>,
}

impl DashboardConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> DashboardConfig {
        DashboardConfig {
            port: self.port.unwrap_or(DEFAULT_PORT),
        }
    }
}

#[derive(Clone)]
pub struct DashboardConfig {
    port: u16,
}

impl DashboardConfig {
    pub fn build() -> DashboardConfigBuilder {
        DashboardConfigBuilder::new()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
