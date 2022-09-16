// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod parameters;
mod receipt;

use alloc::{boxed::Box, vec::Vec};

use derive_more::{Deref, From};
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable};

pub(crate) use self::{parameters::BinaryParametersLength, receipt::ReceiptFundsCount};
pub use self::{
    parameters::ParametersMilestoneOption,
    receipt::{MigratedFundsEntry, ReceiptMilestoneOption, TailTransactionHash},
};
use crate::{protocol::ProtocolParameters, Error};

///
#[derive(Clone, Debug, Eq, PartialEq, From, Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidMilestoneOptionKind)]
#[packable(unpack_visitor = ProtocolParameters)]
pub enum MilestoneOption {
    /// A receipt milestone option.
    #[packable(tag = ReceiptMilestoneOption::KIND)]
    Receipt(ReceiptMilestoneOption),
    /// A parameters milestone option.
    #[packable(tag = ParametersMilestoneOption::KIND)]
    Parameters(ParametersMilestoneOption),
}

impl MilestoneOption {
    /// Return the milestone option kind of a [`MilestoneOption`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Receipt(_) => ReceiptMilestoneOption::KIND,
            Self::Parameters(_) => ParametersMilestoneOption::KIND,
        }
    }
}

pub(crate) type MilestoneOptionCount = BoundedU8<0, { MilestoneOptions::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Deref, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidMilestoneOptionCount(p.into())))]
#[packable(unpack_visitor = ProtocolParameters)]
pub struct MilestoneOptions(
    #[packable(verify_with = verify_unique_sorted_packable)] BoxedSlicePrefix<MilestoneOption, MilestoneOptionCount>,
);

impl TryFrom<Vec<MilestoneOption>> for MilestoneOptions {
    type Error = Error;

    #[inline(always)]
    fn try_from(milestone_options: Vec<MilestoneOption>) -> Result<Self, Self::Error> {
        Self::new(milestone_options)
    }
}

impl IntoIterator for MilestoneOptions {
    type Item = MilestoneOption;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(Into::<Box<[MilestoneOption]>>::into(self.0)).into_iter()
    }
}

impl MilestoneOptions {
    ///
    pub const COUNT_MAX: u8 = 2;

    /// Creates a new [`MilestoneOptions`].
    pub fn new(milestone_options: Vec<MilestoneOption>) -> Result<Self, Error> {
        let mut milestone_options =
            BoxedSlicePrefix::<MilestoneOption, MilestoneOptionCount>::try_from(milestone_options.into_boxed_slice())
                .map_err(Error::InvalidMilestoneOptionCount)?;

        milestone_options.sort_by_key(MilestoneOption::kind);
        // Sort is obviously fine now but uniqueness still needs to be checked.
        verify_unique_sorted::<true>(&milestone_options)?;

        Ok(Self(milestone_options))
    }

    /// Gets a reference to a [`MilestoneOption`] from a milestone option kind, if any.
    #[inline(always)]
    pub fn get(&self, key: u8) -> Option<&MilestoneOption> {
        self.0
            .binary_search_by_key(&key, MilestoneOption::kind)
            // PANIC: indexation is fine since the index has been found.
            .map(|index| &self.0[index])
            .ok()
    }

    /// Gets a reference to a [`ReceiptMilestoneOption`], if any.
    pub fn receipt(&self) -> Option<&ReceiptMilestoneOption> {
        if let Some(MilestoneOption::Receipt(receipt)) = self.get(ReceiptMilestoneOption::KIND) {
            Some(receipt)
        } else {
            None
        }
    }

    /// Gets a reference to a [`ParametersMilestoneOption`], if any.
    pub fn parameters(&self) -> Option<&ParametersMilestoneOption> {
        if let Some(MilestoneOption::Parameters(parameters)) = self.get(ParametersMilestoneOption::KIND) {
            Some(parameters)
        } else {
            None
        }
    }
}

#[inline]
fn verify_unique_sorted<const VERIFY: bool>(milestone_options: &[MilestoneOption]) -> Result<(), Error> {
    if VERIFY && !is_unique_sorted(milestone_options.iter().map(MilestoneOption::kind)) {
        Err(Error::MilestoneOptionsNotUniqueSorted)
    } else {
        Ok(())
    }
}

#[inline]
fn verify_unique_sorted_packable<const VERIFY: bool>(
    milestone_options: &[MilestoneOption],
    _visitor: &ProtocolParameters,
) -> Result<(), Error> {
    verify_unique_sorted::<VERIFY>(milestone_options)
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    pub use self::{
        parameters::dto::ParametersMilestoneOptionDto,
        receipt::dto::{try_from_receipt_milestone_option_dto_for_receipt_milestone_option, ReceiptMilestoneOptionDto},
    };
    use super::*;
    use crate::error::dto::DtoError;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum MilestoneOptionDto {
        /// A receipt milestone option.
        Receipt(ReceiptMilestoneOptionDto),
        /// A parameters milestone option.
        Parameters(ParametersMilestoneOptionDto),
    }

    impl<'de> Deserialize<'de> for MilestoneOptionDto {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;
            Ok(
                match value
                    .get("type")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| serde::de::Error::custom("invalid milestone option type"))?
                    as u8
                {
                    ReceiptMilestoneOption::KIND => {
                        MilestoneOptionDto::Receipt(ReceiptMilestoneOptionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize receipt milestone option: {}", e))
                        })?)
                    }
                    ParametersMilestoneOption::KIND => MilestoneOptionDto::Parameters(
                        ParametersMilestoneOptionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize parameters milestone option: {}", e))
                        })?,
                    ),
                    _ => return Err(serde::de::Error::custom("invalid milestone option type")),
                },
            )
        }
    }

    impl Serialize for MilestoneOptionDto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum MilestoneOptionDto_<'a> {
                T1(&'a ReceiptMilestoneOptionDto),
                T2(&'a ParametersMilestoneOptionDto),
            }
            #[derive(Serialize)]
            struct TypedMilestoneOption<'a> {
                #[serde(flatten)]
                milestone_option: MilestoneOptionDto_<'a>,
            }
            let milestone_option = match self {
                MilestoneOptionDto::Receipt(o) => TypedMilestoneOption {
                    milestone_option: MilestoneOptionDto_::T1(o),
                },
                MilestoneOptionDto::Parameters(o) => TypedMilestoneOption {
                    milestone_option: MilestoneOptionDto_::T2(o),
                },
            };
            milestone_option.serialize(serializer)
        }
    }

    impl From<&MilestoneOption> for MilestoneOptionDto {
        fn from(value: &MilestoneOption) -> Self {
            match value {
                MilestoneOption::Receipt(v) => Self::Receipt(v.into()),
                MilestoneOption::Parameters(v) => Self::Parameters(v.into()),
            }
        }
    }

    pub fn try_from_milestone_option_dto_for_milestone_option(
        value: &MilestoneOptionDto,
        token_supply: u64,
    ) -> Result<MilestoneOption, DtoError> {
        Ok(match value {
            MilestoneOptionDto::Receipt(v) => MilestoneOption::Receipt(
                try_from_receipt_milestone_option_dto_for_receipt_milestone_option(v, token_supply)?,
            ),
            MilestoneOptionDto::Parameters(v) => MilestoneOption::Parameters(v.try_into()?),
        })
    }

    impl MilestoneOptionDto {
        /// Returns the milestone option kind of a [`MilestoneOptionDto`].
        pub fn kind(&self) -> u8 {
            match self {
                Self::Receipt(_) => ReceiptMilestoneOption::KIND,
                Self::Parameters(_) => ParametersMilestoneOption::KIND,
            }
        }
    }
}
