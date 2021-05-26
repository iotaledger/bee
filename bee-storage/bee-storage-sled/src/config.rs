// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

const DEFAULT_FETCH_EDGE_LIMIT: usize = 1_000;
const DEFAULT_FETCH_INDEX_LIMIT: usize = 1_000;
const DEFAULT_FETCH_OUTPUT_ID_LIMIT: usize = 1_000;
const DEFAULT_ITERATION_BUDGET: usize = 100;

#[derive(Clone)]
pub struct StorageConfig {
    pub(crate) fetch_edge_limit: usize,
    pub(crate) fetch_index_limit: usize,
    pub(crate) fetch_output_id_limit: usize,
    pub(crate) iteration_budget: usize,
}

impl From<StorageConfigBuilder> for StorageConfig {
    fn from(builder: StorageConfigBuilder) -> Self {
        builder.finish()
    }
}

#[derive(Default, Deserialize)]
pub struct StorageConfigBuilder {
    fetch_edge_limit: Option<usize>,
    fetch_index_limit: Option<usize>,
    fetch_output_id_limit: Option<usize>,
    iteration_budget: Option<usize>,
}

impl StorageConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> StorageConfig {
        StorageConfig {
            fetch_edge_limit: self.fetch_edge_limit.unwrap_or(DEFAULT_FETCH_EDGE_LIMIT),
            fetch_index_limit: self.fetch_index_limit.unwrap_or(DEFAULT_FETCH_INDEX_LIMIT),
            fetch_output_id_limit: self.fetch_output_id_limit.unwrap_or(DEFAULT_FETCH_OUTPUT_ID_LIMIT),
            iteration_budget: self.iteration_budget.unwrap_or(DEFAULT_ITERATION_BUDGET),
        }
    }
}
