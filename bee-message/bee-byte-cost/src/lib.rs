// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use bee_byte_cost_derive::ByteCost;
pub use packable::Packable;

/// TODO: The weights should allow fractional values while avoiding rounding errors.
#[derive(Clone)]
pub struct ByteCostConfig {
    /// Cost in IOTA coins per virtual byte.
    pub byte_cost: u64,
    /// The weight factor used for key fields in the ouputs.
    pub weight_for_key: u64,
    /// The weight factor used for data fields in the ouputs.
    pub weight_for_data: u64,
}

impl Default for ByteCostConfig {
    fn default() -> Self {
        Self {
            byte_cost: 1,
            weight_for_key: 10,
            weight_for_data: 1,
        }
    }
}

/// A trait to facilitate the computation of the byte cost of message outputs, which is central to dust protection.
/// To be used in combination with [`bee_byte_cost_derive`].
pub trait ByteCost {
    /// Different fields in a type lead to different storage requirements on the tangle.
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
