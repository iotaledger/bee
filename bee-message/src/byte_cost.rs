// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::Output;

use packable::PackableExt;

/// Specifies the current parameters for the byte cost computation.
#[derive(Clone)]
pub struct ByteCostConfig {
    /// Cost in tokens coins per virtual byte.
    pub byte_cost: f32,
    /// The weight factor used for key fields in the ouputs.
    pub weight_for_key: u64,
    /// The weight factor used for data fields in the ouputs.
    pub weight_for_data: u64,
}

impl Default for ByteCostConfig {
    fn default() -> Self {
        Self {
            byte_cost: 1.0,
            weight_for_key: 10,
            weight_for_data: 1,
        }
    }
}

/// A trait to facilitate the computation of the byte cost of message outputs, which is central to dust protection.
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
    (config.byte_cost * output.weighted_bytes(config) as f32) as u64
}

impl ByteCost for Output {
    fn weighted_bytes(&self, config: &ByteCostConfig) -> u64 {
        // The updated verison of TIP19 (https://github.com/iotaledger/tips/pull/39) has been largely simplified.
        // Now, all fields of all outputs are marked as `data. Therefore, we can just resort to using `Packable` to
        // compute the `weighted bytes`.
        self.packed_len() as u64 * config.weight_for_data
    }
}
