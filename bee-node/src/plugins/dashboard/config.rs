// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct DashboardConfigBuilder {}

impl DashboardConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> DashboardConfig {
        DashboardConfig {}
    }
}

#[derive(Clone)]
pub struct DashboardConfig {}

impl DashboardConfig {
    pub fn build() -> DashboardConfigBuilder {
        DashboardConfigBuilder::new()
    }
}
