// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod address;
mod dust_deposit_return;
mod expiration;
mod governor_address;
mod state_controller_address;
mod timelock;

pub use address::AddressUnlockCondition;
pub(crate) use dust_deposit_return::DustDepositAmount;
pub use dust_deposit_return::DustDepositReturnUnlockCondition;
pub use expiration::ExpirationUnlockCondition;
pub use governor_address::GovernorAddressUnlockCondition;
pub use state_controller_address::StateControllerAddressUnlockCondition;
pub use timelock::TimelockUnlockCondition;

use crate::Error;

use bee_common::ord::is_unique_sorted;

use bitflags::bitflags;
use derive_more::{Deref, From};
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidUnlockConditionKind)]
pub enum UnlockCondition {
    /// An address unlock condition.
    #[packable(tag = AddressUnlockCondition::KIND)]
    Address(AddressUnlockCondition),
    /// A dust deposit return unlock condition.
    #[packable(tag = DustDepositReturnUnlockCondition::KIND)]
    DustDepositReturn(DustDepositReturnUnlockCondition),
    /// A timelock unlock condition.
    #[packable(tag = TimelockUnlockCondition::KIND)]
    Timelock(TimelockUnlockCondition),
    /// An expiration unlock condition.
    #[packable(tag = ExpirationUnlockCondition::KIND)]
    Expiration(ExpirationUnlockCondition),
    /// A state controller address unlock condition.
    #[packable(tag = StateControllerAddressUnlockCondition::KIND)]
    StateControllerAddress(StateControllerAddressUnlockCondition),
    /// A governor address unlock condition.
    #[packable(tag = GovernorAddressUnlockCondition::KIND)]
    GovernorAddress(GovernorAddressUnlockCondition),
}

impl UnlockCondition {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Address(_) => AddressUnlockCondition::KIND,
            Self::DustDepositReturn(_) => DustDepositReturnUnlockCondition::KIND,
            Self::Timelock(_) => TimelockUnlockCondition::KIND,
            Self::Expiration(_) => ExpirationUnlockCondition::KIND,
            Self::StateControllerAddress(_) => StateControllerAddressUnlockCondition::KIND,
            Self::GovernorAddress(_) => GovernorAddressUnlockCondition::KIND,
        }
    }

    /// Returns the [`UnlockConditionFlags`] for the given [`UnlockCondition`].
    pub(crate) fn flag(&self) -> UnlockConditionFlags {
        match self {
            Self::Address(_) => UnlockConditionFlags::ADDRESS,
            Self::DustDepositReturn(_) => UnlockConditionFlags::DUST_DEPOSIT_RETURN,
            Self::Timelock(_) => UnlockConditionFlags::TIMELOCK,
            Self::Expiration(_) => UnlockConditionFlags::EXPIRATION,
            Self::StateControllerAddress(_) => UnlockConditionFlags::STATE_CONTROLLER_ADDRESS,
            Self::GovernorAddress(_) => UnlockConditionFlags::GOVERNOR_ADDRESS,
        }
    }
}

pub(crate) type UnlockConditionCount = BoundedU8<0, { UnlockConditions::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deref, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_packable_or_else(|p| Error::InvalidUnlockConditionCount(p.into())))]
pub struct UnlockConditions(
    #[packable(verify_with = Self::validate_unlock_conditions)] BoxedSlicePrefix<UnlockCondition, UnlockConditionCount>,
);

impl TryFrom<Vec<UnlockCondition>> for UnlockConditions {
    type Error = Error;

    #[inline(always)]
    fn try_from(unlock_conditions: Vec<UnlockCondition>) -> Result<Self, Self::Error> {
        Self::new(unlock_conditions)
    }
}

impl UnlockConditions {
    ///
    pub const COUNT_MAX: u8 = 3;

    /// Creates a new `UnlockConditions`.
    pub fn new(unlock_conditions: Vec<UnlockCondition>) -> Result<Self, Error> {
        let mut unlock_conditions =
            BoxedSlicePrefix::<UnlockCondition, UnlockConditionCount>::try_from(unlock_conditions.into_boxed_slice())
                .map_err(Error::InvalidUnlockConditionCount)?;

        unlock_conditions.sort_by_key(UnlockCondition::kind);
        Self::validate_unlock_conditions::<true>(&unlock_conditions)?;

        Ok(Self(unlock_conditions))
    }

    fn validate_unlock_conditions<const VERIFY: bool>(unlock_conditions: &[UnlockCondition]) -> Result<(), Error> {
        if VERIFY {
            // Sort is obviously fine now but uniqueness still needs to be checked.
            validate_unique_sorted(unlock_conditions)
        } else {
            Ok(())
        }
    }

    /// Gets a reference to a unlock condition from a unlock condition kind, if found.
    #[inline(always)]
    pub fn get(&self, key: u8) -> Option<&UnlockCondition> {
        self.0
            .binary_search_by_key(&key, UnlockCondition::kind)
            // SAFETY: indexation is fine since the index has been found.
            .map(|index| &self.0[index])
            .ok()
    }

    /// Returns the length of the unlock conditions.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the [`UnlockConditions`] is empty or not.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[inline]
fn validate_unique_sorted(unlock_conditions: &[UnlockCondition]) -> Result<(), Error> {
    if !is_unique_sorted(unlock_conditions.iter().map(UnlockCondition::kind)) {
        return Err(Error::UnlockConditionsNotUniqueSorted);
    }

    Ok(())
}

pub(crate) fn validate_allowed_unlock_conditions(
    unlock_conditions: &UnlockConditions,
    allowed_unlock_conditions: UnlockConditionFlags,
) -> Result<(), Error> {
    for (index, unlock_condition) in unlock_conditions.iter().enumerate() {
        if !allowed_unlock_conditions.contains(unlock_condition.flag()) {
            return Err(Error::UnallowedUnlockCondition {
                index,
                kind: unlock_condition.kind(),
            });
        }
    }
    Ok(())
}

bitflags! {
    /// A bitflags-based representation of the set of active unlock conditions.
    pub(crate) struct UnlockConditionFlags: u16 {
        /// Signals the presence of an [`AddressUnlockCondition`].
        const ADDRESS = 1 << AddressUnlockCondition::KIND;
        /// Signals the presence of a [`DustDepositReturnUnlockCondition`].
        const DUST_DEPOSIT_RETURN = 1 << DustDepositReturnUnlockCondition::KIND;
        /// Signals the presence of a [`TimelockUnlockCondition`].
        const TIMELOCK = 1 << TimelockUnlockCondition::KIND;
        /// Signals the presence of a [`ExpirationUnlockCondition`].
        const EXPIRATION = 1 << ExpirationUnlockCondition::KIND;
        /// Signals the presence of a [`StateControllerAddressUnlockCondition`].
        const STATE_CONTROLLER_ADDRESS = 1 << StateControllerAddressUnlockCondition::KIND;
        /// Signals the presence of a [`GovernorAddressUnlockCondition`].
        const GOVERNOR_ADDRESS = 1 << GovernorAddressUnlockCondition::KIND;
    }
}
