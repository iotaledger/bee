// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Builder for a [`TangleConfig`].
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize), serde(rename_all = "camelCase"))]
pub struct TangleConfigBuilder {}

impl TangleConfigBuilder {
    /// Creates a new [`TangleConfigBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Finishes a [`TangleConfigBuilder`] into a [`TangleConfig`].
    pub fn finish(self) -> TangleConfig {
        TangleConfig {}
    }
}

/// Configuration for a tangle.
#[derive(Clone)]
pub struct TangleConfig {}

impl Default for TangleConfig {
    fn default() -> TangleConfig {
        TangleConfigBuilder::default().finish()
    }
}

impl TangleConfig {
    /// Creates a new [`TangleConfigBuilder`].
    pub fn build() -> TangleConfigBuilder {
        TangleConfigBuilder::default()
    }
}
