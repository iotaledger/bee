// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the parameters milestone option.

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use packable::{bounded::BoundedU16, prefix::BoxedSlicePrefix, Packable};

use crate::{payload::milestone::MilestoneIndex, Error};

pub(crate) type BinaryParametersLength = BoundedU16<
    { *ParametersMilestoneOption::BINARY_PARAMETERS_LENGTH_RANGE.start() },
    { *ParametersMilestoneOption::BINARY_PARAMETERS_LENGTH_RANGE.end() },
>;

///
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct ParametersMilestoneOption {
    // The milestone index at which these protocol parameters become active.
    target_milestone_index: MilestoneIndex,
    // The protocol version.
    protocol_version: u8,
    // The protocol parameters in binary form.
    #[packable(unpack_error_with = |err| Error::InvalidBinaryParametersLength(err.into_prefix_err().into()))]
    binary_parameters: BoxedSlicePrefix<u8, BinaryParametersLength>,
}

impl ParametersMilestoneOption {
    /// The milestone option kind of a [`ParametersMilestoneOption`].
    pub const KIND: u8 = 1;
    /// Valid lengths for binary parameters.
    pub const BINARY_PARAMETERS_LENGTH_RANGE: RangeInclusive<u16> = 0..=8192;

    /// Creates a new [`ParametersMilestoneOption`].
    pub fn new(
        target_milestone_index: MilestoneIndex,
        protocol_version: u8,
        binary_parameters: Vec<u8>,
    ) -> Result<Self, Error> {
        Ok(Self {
            target_milestone_index,
            protocol_version,
            binary_parameters: binary_parameters
                .into_boxed_slice()
                .try_into()
                .map_err(Error::InvalidBinaryParametersLength)?,
        })
    }

    /// Returns the target milestone index of a [`ParametersMilestoneOption`].
    pub fn target_milestone_index(&self) -> MilestoneIndex {
        self.target_milestone_index
    }

    /// Returns the protocol version of a [`ParametersMilestoneOption`].
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version
    }

    /// Returns the binary parameters of a [`ParametersMilestoneOption`].
    pub fn binary_parameters(&self) -> &[u8] {
        &self.binary_parameters
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    ///
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ParametersMilestoneOptionDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "targetMilestoneIndex")]
        pub target_milestone_index: u32,
        #[serde(rename = "protocolVersion")]
        pub protocol_version: u8,
        #[serde(rename = "params")]
        pub binary_parameters: String,
    }

    impl From<&ParametersMilestoneOption> for ParametersMilestoneOptionDto {
        fn from(value: &ParametersMilestoneOption) -> Self {
            ParametersMilestoneOptionDto {
                kind: ParametersMilestoneOption::KIND,
                target_milestone_index: *value.target_milestone_index(),
                protocol_version: value.protocol_version(),
                binary_parameters: prefix_hex::encode(value.binary_parameters()),
            }
        }
    }

    impl TryFrom<&ParametersMilestoneOptionDto> for ParametersMilestoneOption {
        type Error = DtoError;

        fn try_from(value: &ParametersMilestoneOptionDto) -> Result<Self, Self::Error> {
            Ok(ParametersMilestoneOption::new(
                value.target_milestone_index.into(),
                value.protocol_version,
                prefix_hex::decode(&value.binary_parameters).map_err(|_| DtoError::InvalidField("params"))?,
            )?)
        }
    }
}
