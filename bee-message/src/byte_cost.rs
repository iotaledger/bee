// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const DEFAULT_BYTE_COST: u64 = 500;
const DEFAULT_BYTE_COST_KEY_WEIGHT: u64 = 10;
const DEFAULT_BYTE_COST_DATA_WEIGHT: u64 = 1;

/// Builder for a [`ByteCostConfig`].
#[derive(Default)]
#[cfg_attr(feature = "serde1", derive(serde::Deserialize))]
#[must_use]
pub struct ByteCostConfigBuilder {
    v_byte_cost: Option<u64>,
    v_byte_factor_key: Option<u64>,
    v_byte_factor_data: Option<u64>,
}

impl ByteCostConfigBuilder {
    /// Returns a new [`ByteCostConfigBuilder`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the byte cost for dust protection.
    pub fn byte_cost(mut self, byte_cost: u64) -> Self {
        self.v_byte_cost.replace(byte_cost);
        self
    }

    /// Sets the byte cost for dust protection.
    pub fn weight_for_key_field(mut self, weight: u64) -> Self {
        self.v_byte_factor_key.replace(weight);
        self
    }

    /// Sets the byte cost for dust protection.
    pub fn weight_for_data_field(mut self, weight: u64) -> Self {
        self.v_byte_factor_data.replace(weight);
        self
    }

    /// Returns the built [`ByteCostConfig`].
    pub fn finish(self) -> ByteCostConfig {
        ByteCostConfig {
            v_byte_cost: self.v_byte_cost.unwrap_or(DEFAULT_BYTE_COST),
            v_byte_factor_key: self.v_byte_factor_key.unwrap_or(DEFAULT_BYTE_COST_KEY_WEIGHT),
            v_byte_factor_data: self.v_byte_factor_data.unwrap_or(DEFAULT_BYTE_COST_DATA_WEIGHT),
        }
    }
}

/// Specifies the current parameters for the byte cost computation.
#[derive(Clone)]
pub struct ByteCostConfig {
    /// Cost in tokens coins per virtual byte.
    pub v_byte_cost: u64,
    /// The weight factor used for key fields in the ouputs.
    pub v_byte_factor_key: u64,
    /// The weight factor used for data fields in the ouputs.
    pub v_byte_factor_data: u64,
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
    config.v_byte_cost * output.weighted_bytes(config)
}
