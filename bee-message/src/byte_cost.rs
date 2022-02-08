// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::Output;

use packable::PackableExt;
use serde::Deserialize;

const DEFAULT_BYTE_COST: u64 = 500;
const DEFAULT_BYTE_COST_KEY_WEIGHT: u64 = 10;
const DEFAULT_BYTE_COST_DATA_WEIGHT: u64 = 1;

/// Builder for a [`ByteCostConfig`].
#[derive(Default, Deserialize)]
#[must_use]
pub struct ByteCostConfigBuilder {
    byte_cost: Option<u64>,
    weight_for_key: Option<u64>,
    weight_for_data: Option<u64>,
}

impl ByteCostConfigBuilder {
    /// Returns a new [`ByteCostConfigBuilder`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the byte cost for dust protection.
    pub fn byte_cost(mut self, byte_cost: u64) -> Self {
        self.byte_cost.replace(byte_cost);
        self
    }

    /// Sets the byte cost for dust protection.
    pub fn weight_for_key_field(mut self, weight: u64) -> Self {
        self.weight_for_key.replace(weight);
        self
    }

    /// Sets the byte cost for dust protection.
    pub fn weight_for_data_field(mut self, weight: u64) -> Self {
        self.weight_for_key.replace(weight);
        self
    }

    /// Returns the built [`ByteCostConfig`].
    pub fn finish(self) -> ByteCostConfig {
        ByteCostConfig {
            byte_cost: self.byte_cost.unwrap_or(DEFAULT_BYTE_COST),
            weight_for_key: self.weight_for_key.unwrap_or(DEFAULT_BYTE_COST_KEY_WEIGHT),
            weight_for_data: self.weight_for_data.unwrap_or(DEFAULT_BYTE_COST_DATA_WEIGHT),
        }
    }
}

/// Specifies the current parameters for the byte cost computation.
#[derive(Clone)]
pub struct ByteCostConfig {
    /// Cost in tokens coins per virtual byte.
    pub byte_cost: u64,
    /// The weight factor used for key fields in the ouputs.
    pub weight_for_key: u64,
    /// The weight factor used for data fields in the ouputs.
    pub weight_for_data: u64,
}

impl ByteCostConfig {
    /// Returns a builder for this config.
    pub fn build() -> ByteCostConfigBuilder {
        ByteCostConfigBuilder::new()
    }
}

/// A trait to facilitate the computation of the byte cost of message outputs, which is central to dust protection.
pub trait ByteCost {
    /// Different fields in a type lead to different storage requirements for the ledger state.
    fn weighted_bytes(&self, config: &ByteCostConfig) -> u64;
}

impl<T: ByteCost, const N: usize> ByteCost for [T; N] {
    fn weighted_bytes(&self, config: &ByteCostConfig) -> u64 {
        self.iter().map(|elem| elem.weighted_bytes(config)).sum()
    }
}

/// Computes the storage cost for `[crate::output::Output]`s.
pub fn min_deposit(config: &ByteCostConfig, output: &impl ByteCost) -> u64 {
    config.byte_cost * output.weighted_bytes(config)
}

impl ByteCost for Output {
    fn weighted_bytes(&self, config: &ByteCostConfig) -> u64 {
        // The updated verison of TIP19 (https://github.com/iotaledger/tips/pull/39) has been largely simplified.
        // Now, all fields of all outputs are marked as `data. Therefore, we can just resort to using `Packable` to
        // compute the `weighted bytes`.
        self.packed_len() as u64 * config.weight_for_data
    }
}
