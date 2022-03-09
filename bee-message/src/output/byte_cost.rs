// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, output::OutputId, MessageId};

use core::mem::size_of;

const DEFAULT_BYTE_COST: u64 = 500;
const DEFAULT_BYTE_COST_FACTOR_KEY: u64 = 10;
const DEFAULT_BYTE_COST_FACTOR_DATA: u64 = 1;

type ConfirmationUnixTimestamp = u32;

/// Builder for a [`ByteCostConfig`].
#[derive(Default, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Deserialize))]
#[must_use]
pub struct ByteCostConfigBuilder {
    #[cfg_attr(feature = "serde1", serde(alias = "vByteCost"))]
    v_byte_cost: Option<u64>,
    #[cfg_attr(feature = "serde1", serde(alias = "vByteFactorKey"))]
    v_byte_factor_key: Option<u64>,
    #[cfg_attr(feature = "serde1", serde(alias = "vByteFactorData"))]
    v_byte_factor_data: Option<u64>,
}

impl ByteCostConfigBuilder {
    /// Returns a new [`ByteCostConfigBuilder`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the byte cost for the storage deposit.
    pub fn byte_cost(mut self, byte_cost: u64) -> Self {
        self.v_byte_cost.replace(byte_cost);
        self
    }

    /// Sets the virtual byte weight for the key fields.
    pub fn key_factor(mut self, weight: u64) -> Self {
        self.v_byte_factor_key.replace(weight);
        self
    }

    /// Sets the virtual byte weight for the data fields.
    pub fn data_factor(mut self, weight: u64) -> Self {
        self.v_byte_factor_data.replace(weight);
        self
    }

    /// Returns the built [`ByteCostConfig`].
    pub fn finish(self) -> ByteCostConfig {
        let v_byte_factor_key = self.v_byte_factor_key.unwrap_or(DEFAULT_BYTE_COST_FACTOR_KEY);
        let v_byte_factor_data = self.v_byte_factor_data.unwrap_or(DEFAULT_BYTE_COST_FACTOR_DATA);

        let v_byte_offset = size_of::<OutputId>() as u64 * v_byte_factor_key
            + size_of::<MessageId>() as u64 * v_byte_factor_data
            + size_of::<MilestoneIndex>() as u64 * v_byte_factor_data
            + size_of::<ConfirmationUnixTimestamp>() as u64 * v_byte_factor_data;

        ByteCostConfig {
            v_byte_cost: self.v_byte_cost.unwrap_or(DEFAULT_BYTE_COST),
            v_byte_factor_key,
            v_byte_factor_data,
            v_byte_offset,
        }
    }
}

/// Specifies the current parameters for the byte cost computation.
#[derive(Clone)]
pub struct ByteCostConfig {
    /// Cost in tokens per virtual byte.
    pub v_byte_cost: u64,
    /// The weight factor used for key fields in the ouputs.
    pub v_byte_factor_key: u64,
    /// The weight factor used for data fields in the ouputs.
    pub v_byte_factor_data: u64,
    /// The offset in addition to the other fields.
    v_byte_offset: u64,
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

    /// Computes the byte cost given a [`ByteCostConfig`].
    fn byte_cost(&self, config: &ByteCostConfig) -> u64 {
        config.v_byte_cost * (self.weighted_bytes(config) + config.v_byte_offset)
    }
}

impl<T: ByteCost, const N: usize> ByteCost for [T; N] {
    fn weighted_bytes(&self, config: &ByteCostConfig) -> u64 {
        self.iter().map(|elem| elem.weighted_bytes(config)).sum()
    }
}

#[cfg(test)]
mod test {
    use crate::output::{ByteCost, ByteCostConfig, ByteCostConfigBuilder, Output};

    use bee_test::rand::output::{rand_alias_output, rand_basic_output, rand_foundry_output, rand_nft_output};

    use packable::{Packable, PackableExt};

    const BYTE_COST: u64 = 1;
    const FACTOR_KEY: u64 = 10;
    const FACTOR_DATA: u64 = 1;

    fn config() -> ByteCostConfig {
        ByteCostConfigBuilder::new()
            .byte_cost(BYTE_COST)
            .key_factor(FACTOR_KEY)
            .data_factor(FACTOR_DATA)
            .finish()
    }

    // We have to jump through hoops here because the randomly generated outputs from `bee_test` have a different
    // type then the Outputs of this crate (although they are technically the same).
    fn convert<T: PackableExt>(rand_output: impl Packable) -> T {
        let bytes = rand_output.pack_to_vec();
        // Safety: We know it's the right type.
        T::unpack_unverified(bytes).unwrap()
    }

    fn output_in_range(output: Output, range: std::ops::RangeInclusive<u64>) {
        let cost = output.byte_cost(&config());
        assert!(
            range.contains(&cost),
            "{:#?} has a required byte cost of {}",
            output,
            cost
        );
    }

    #[test]
    fn valid_byte_cost_range() {
        output_in_range(Output::Alias(convert(rand_alias_output())), 445..=29_620);
        output_in_range(Output::Basic(convert(rand_basic_output())), 414..=13_485);
        output_in_range(Output::Foundry(convert(rand_foundry_output())), 496..=21_365);
        output_in_range(Output::Nft(convert(rand_nft_output())), 435..=21_734);
    }
}
