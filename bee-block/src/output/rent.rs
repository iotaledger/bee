// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::mem::size_of;

use crate::{output::OutputId, payload::milestone::MilestoneIndex, BlockId};

const DEFAULT_BYTE_COST: u32 = 500;
const DEFAULT_BYTE_COST_FACTOR_KEY: u8 = 10;
const DEFAULT_BYTE_COST_FACTOR_DATA: u8 = 1;

type ConfirmationUnixTimestamp = u32;

/// Builder for a [`RentStructure`].
#[derive(Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[must_use]
pub struct RentStructureBuilder {
    #[cfg_attr(feature = "serde", serde(alias = "vByteCost"))]
    v_byte_cost: Option<u32>,
    #[cfg_attr(feature = "serde", serde(alias = "vByteFactorKey"))]
    v_byte_factor_key: Option<u8>,
    #[cfg_attr(feature = "serde", serde(alias = "vByteFactorData"))]
    v_byte_factor_data: Option<u8>,
}

impl RentStructureBuilder {
    /// Returns a new [`RentStructureBuilder`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the byte cost for the storage deposit.
    pub fn byte_cost(mut self, byte_cost: u32) -> Self {
        self.v_byte_cost.replace(byte_cost);
        self
    }

    /// Sets the virtual byte weight for the key fields.
    pub fn key_factor(mut self, weight: u8) -> Self {
        self.v_byte_factor_key.replace(weight);
        self
    }

    /// Sets the virtual byte weight for the data fields.
    pub fn data_factor(mut self, weight: u8) -> Self {
        self.v_byte_factor_data.replace(weight);
        self
    }

    /// Returns the built [`RentStructure`].
    pub fn finish(self) -> RentStructure {
        let v_byte_factor_key = self.v_byte_factor_key.unwrap_or(DEFAULT_BYTE_COST_FACTOR_KEY);
        let v_byte_factor_data = self.v_byte_factor_data.unwrap_or(DEFAULT_BYTE_COST_FACTOR_DATA);

        let v_byte_offset = size_of::<OutputId>() as u32 * v_byte_factor_key as u32
            + size_of::<BlockId>() as u32 * v_byte_factor_data as u32
            + size_of::<MilestoneIndex>() as u32 * v_byte_factor_data as u32
            + size_of::<ConfirmationUnixTimestamp>() as u32 * v_byte_factor_data as u32;

        RentStructure {
            v_byte_cost: self.v_byte_cost.unwrap_or(DEFAULT_BYTE_COST),
            v_byte_factor_key,
            v_byte_factor_data,
            v_byte_offset,
        }
    }
}

/// Specifies the current parameters for the byte cost computation.
#[derive(Clone)]
pub struct RentStructure {
    /// Cost in tokens per virtual byte.
    pub v_byte_cost: u32,
    /// The weight factor used for key fields in the ouputs.
    pub v_byte_factor_key: u8,
    /// The weight factor used for data fields in the ouputs.
    pub v_byte_factor_data: u8,
    /// The offset in addition to the other fields.
    v_byte_offset: u32,
}

impl RentStructure {
    /// Returns a builder for this config.
    pub fn build() -> RentStructureBuilder {
        RentStructureBuilder::new()
    }
}

/// A trait to facilitate the computation of the byte cost of block outputs, which is central to dust protection.
pub trait Rent {
    /// Different fields in a type lead to different storage requirements for the ledger state.
    fn weighted_bytes(&self, config: &RentStructure) -> u64;

    /// Computes the rent cost given a [`RentStructure`].
    fn rent_cost(&self, config: &RentStructure) -> u64 {
        config.v_byte_cost as u64 * (self.weighted_bytes(config) + config.v_byte_offset as u64)
    }
}

impl<T: Rent, const N: usize> Rent for [T; N] {
    fn weighted_bytes(&self, config: &RentStructure) -> u64 {
        self.iter().map(|elem| elem.weighted_bytes(config)).sum()
    }
}
