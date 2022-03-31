// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod address;
mod expiration;
mod governor_address;
mod immutable_alias_address;
mod state_controller_address;
mod storage_deposit_return;
mod timelock;

use alloc::vec::Vec;

use bitflags::bitflags;
use derive_more::{Deref, From};
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable};

pub(crate) use self::storage_deposit_return::StorageDepositAmount;
pub use self::{
    address::AddressUnlockCondition, expiration::ExpirationUnlockCondition,
    governor_address::GovernorAddressUnlockCondition, immutable_alias_address::ImmutableAliasAddressUnlockCondition,
    state_controller_address::StateControllerAddressUnlockCondition,
    storage_deposit_return::StorageDepositReturnUnlockCondition, timelock::TimelockUnlockCondition,
};
use crate::{create_bitflags, Error};

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
    /// A storage deposit return unlock condition.
    #[packable(tag = StorageDepositReturnUnlockCondition::KIND)]
    StorageDepositReturn(StorageDepositReturnUnlockCondition),
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
    /// An immutable alias address unlock condition.
    #[packable(tag = ImmutableAliasAddressUnlockCondition::KIND)]
    ImmutableAliasAddress(ImmutableAliasAddressUnlockCondition),
}

impl UnlockCondition {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Address(_) => AddressUnlockCondition::KIND,
            Self::StorageDepositReturn(_) => StorageDepositReturnUnlockCondition::KIND,
            Self::Timelock(_) => TimelockUnlockCondition::KIND,
            Self::Expiration(_) => ExpirationUnlockCondition::KIND,
            Self::StateControllerAddress(_) => StateControllerAddressUnlockCondition::KIND,
            Self::GovernorAddress(_) => GovernorAddressUnlockCondition::KIND,
            Self::ImmutableAliasAddress(_) => ImmutableAliasAddressUnlockCondition::KIND,
        }
    }

    /// Returns the [`UnlockConditionFlags`] for the given [`UnlockCondition`].
    pub(crate) fn flag(&self) -> UnlockConditionFlags {
        match self {
            Self::Address(_) => UnlockConditionFlags::ADDRESS,
            Self::StorageDepositReturn(_) => UnlockConditionFlags::STORAGE_DEPOSIT_RETURN,
            Self::Timelock(_) => UnlockConditionFlags::TIMELOCK,
            Self::Expiration(_) => UnlockConditionFlags::EXPIRATION,
            Self::StateControllerAddress(_) => UnlockConditionFlags::STATE_CONTROLLER_ADDRESS,
            Self::GovernorAddress(_) => UnlockConditionFlags::GOVERNOR_ADDRESS,
            Self::ImmutableAliasAddress(_) => UnlockConditionFlags::IMMUTABLE_ALIAS_ADDRESS,
        }
    }
}

create_bitflags!(
    /// A bitflags-based representation of the set of active [`UnlockCondition`]s.
    pub UnlockConditionFlags,
    u16,
    [
        (ADDRESS, AddressUnlockCondition),
        (STORAGE_DEPOSIT_RETURN, StorageDepositReturnUnlockCondition),
        (TIMELOCK, TimelockUnlockCondition),
        (EXPIRATION, ExpirationUnlockCondition),
        (STATE_CONTROLLER_ADDRESS, StateControllerAddressUnlockCondition),
        (GOVERNOR_ADDRESS, GovernorAddressUnlockCondition),
        (IMMUTABLE_ALIAS_ADDRESS, ImmutableAliasAddressUnlockCondition),
    ]
);

pub(crate) type UnlockConditionCount = BoundedU8<0, { UnlockConditions::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deref, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidUnlockConditionCount(p.into())))]
pub struct UnlockConditions(
    #[packable(verify_with = verify_unique_sorted)] BoxedSlicePrefix<UnlockCondition, UnlockConditionCount>,
);

impl TryFrom<Vec<UnlockCondition>> for UnlockConditions {
    type Error = Error;

    #[inline(always)]
    fn try_from(unlock_conditions: Vec<UnlockCondition>) -> Result<Self, Self::Error> {
        Self::new(unlock_conditions)
    }
}

impl IntoIterator for UnlockConditions {
    type Item = UnlockCondition;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(Into::<Box<[UnlockCondition]>>::into(self.0)).into_iter()
    }
}

impl UnlockConditions {
    ///
    pub const COUNT_MAX: u8 = 7;

    /// Creates a new [`UnlockConditions`].
    pub fn new(unlock_conditions: Vec<UnlockCondition>) -> Result<Self, Error> {
        let mut unlock_conditions =
            BoxedSlicePrefix::<UnlockCondition, UnlockConditionCount>::try_from(unlock_conditions.into_boxed_slice())
                .map_err(Error::InvalidUnlockConditionCount)?;

        unlock_conditions.sort_by_key(UnlockCondition::kind);
        // Sort is obviously fine now but uniqueness still needs to be checked.
        verify_unique_sorted::<true>(&unlock_conditions)?;

        Ok(Self(unlock_conditions))
    }

    /// Gets a reference to an [`UnlockCondition`] from an unlock condition kind, if any.
    #[inline(always)]
    pub fn get(&self, key: u8) -> Option<&UnlockCondition> {
        self.0
            .binary_search_by_key(&key, UnlockCondition::kind)
            // PANIC: indexation is fine since the index has been found.
            .map(|index| &self.0[index])
            .ok()
    }

    /// Gets a reference to an [`AddressUnlockCondition`], if any.
    #[inline(always)]
    pub fn address(&self) -> Option<&AddressUnlockCondition> {
        if let Some(UnlockCondition::Address(address)) = self.get(AddressUnlockCondition::KIND) {
            Some(address)
        } else {
            None
        }
    }

    /// Gets a reference to a [`StorageDepositReturnUnlockCondition`], if any.
    #[inline(always)]
    pub fn storage_deposit_return(&self) -> Option<&StorageDepositReturnUnlockCondition> {
        if let Some(UnlockCondition::StorageDepositReturn(storage_deposit_return)) =
            self.get(StorageDepositReturnUnlockCondition::KIND)
        {
            Some(storage_deposit_return)
        } else {
            None
        }
    }

    /// Gets a reference to a [`TimelockUnlockCondition`], if any.
    #[inline(always)]
    pub fn timelock(&self) -> Option<&TimelockUnlockCondition> {
        if let Some(UnlockCondition::Timelock(timelock)) = self.get(TimelockUnlockCondition::KIND) {
            Some(timelock)
        } else {
            None
        }
    }

    /// Gets a reference to an [`ExpirationUnlockCondition`], if any.
    #[inline(always)]
    pub fn expiration(&self) -> Option<&ExpirationUnlockCondition> {
        if let Some(UnlockCondition::Expiration(expiration)) = self.get(ExpirationUnlockCondition::KIND) {
            Some(expiration)
        } else {
            None
        }
    }

    /// Gets a reference to a [`StateControllerAddressUnlockCondition`], if any.
    #[inline(always)]
    pub fn state_controller_address(&self) -> Option<&StateControllerAddressUnlockCondition> {
        if let Some(UnlockCondition::StateControllerAddress(state_controller_address)) =
            self.get(StateControllerAddressUnlockCondition::KIND)
        {
            Some(state_controller_address)
        } else {
            None
        }
    }

    /// Gets a reference to a [`GovernorAddressUnlockCondition`], if any.
    #[inline(always)]
    pub fn governor_address(&self) -> Option<&GovernorAddressUnlockCondition> {
        if let Some(UnlockCondition::GovernorAddress(governor_address)) = self.get(GovernorAddressUnlockCondition::KIND)
        {
            Some(governor_address)
        } else {
            None
        }
    }

    /// Gets a reference to an [`ImmutableAliasAddressUnlockCondition`], if any.
    #[inline(always)]
    pub fn immutable_alias_address(&self) -> Option<&ImmutableAliasAddressUnlockCondition> {
        if let Some(UnlockCondition::ImmutableAliasAddress(immutable_alias_address)) =
            self.get(ImmutableAliasAddressUnlockCondition::KIND)
        {
            Some(immutable_alias_address)
        } else {
            None
        }
    }
}

#[inline]
fn verify_unique_sorted<const VERIFY: bool>(unlock_conditions: &[UnlockCondition]) -> Result<(), Error> {
    if VERIFY && !is_unique_sorted(unlock_conditions.iter().map(UnlockCondition::kind)) {
        Err(Error::UnlockConditionsNotUniqueSorted)
    } else {
        Ok(())
    }
}

pub(crate) fn verify_allowed_unlock_conditions(
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn all_flags_present() {
        assert_eq!(
            UnlockConditionFlags::ALL_FLAGS,
            &[
                UnlockConditionFlags::ADDRESS,
                UnlockConditionFlags::STORAGE_DEPOSIT_RETURN,
                UnlockConditionFlags::TIMELOCK,
                UnlockConditionFlags::EXPIRATION,
                UnlockConditionFlags::STATE_CONTROLLER_ADDRESS,
                UnlockConditionFlags::GOVERNOR_ADDRESS,
                UnlockConditionFlags::IMMUTABLE_ALIAS_ADDRESS
            ]
        );
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    pub use self::{
        address::dto::AddressUnlockConditionDto, expiration::dto::ExpirationUnlockConditionDto,
        governor_address::dto::GovernorAddressUnlockConditionDto,
        immutable_alias_address::dto::ImmutableAliasAddressUnlockConditionDto,
        state_controller_address::dto::StateControllerAddressUnlockConditionDto,
        storage_deposit_return::dto::StorageDepositReturnUnlockConditionDto, timelock::dto::TimelockUnlockConditionDto,
    };
    use super::*;
    use crate::{
        address::{dto::AddressDto, Address},
        error::dto::DtoError,
    };

    #[derive(Clone, Debug)]
    pub enum UnlockConditionDto {
        /// An address unlock condition.
        Address(AddressUnlockConditionDto),
        /// A storage deposit return unlock condition.
        StorageDepositReturn(StorageDepositReturnUnlockConditionDto),
        /// A timelock unlock condition.
        Timelock(TimelockUnlockConditionDto),
        /// An expiration unlock condition.
        Expiration(ExpirationUnlockConditionDto),
        /// A state controller address unlock condition.
        StateControllerAddress(StateControllerAddressUnlockConditionDto),
        /// A governor address unlock condition.
        GovernorAddress(GovernorAddressUnlockConditionDto),
        /// An immutable alias address unlock condition.
        ImmutableAliasAddress(ImmutableAliasAddressUnlockConditionDto),
    }

    impl<'de> Deserialize<'de> for UnlockConditionDto {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;
            Ok(
                match value
                    .get("type")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| serde::de::Error::custom("invalid unlock condition type"))?
                    as u8
                {
                    AddressUnlockCondition::KIND => {
                        UnlockConditionDto::Address(AddressUnlockConditionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize address unlock condition: {}", e))
                        })?)
                    }
                    StorageDepositReturnUnlockCondition::KIND => UnlockConditionDto::StorageDepositReturn(
                        StorageDepositReturnUnlockConditionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!(
                                "cannot deserialize storage deposit unlock condition: {}",
                                e
                            ))
                        })?,
                    ),
                    TimelockUnlockCondition::KIND => {
                        UnlockConditionDto::Timelock(TimelockUnlockConditionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize timelock unlock condition: {}", e))
                        })?)
                    }
                    ExpirationUnlockCondition::KIND => UnlockConditionDto::Expiration(
                        ExpirationUnlockConditionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize expiration unlock condition: {}", e))
                        })?,
                    ),
                    StateControllerAddressUnlockCondition::KIND => UnlockConditionDto::StateControllerAddress(
                        StateControllerAddressUnlockConditionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!(
                                "cannot deserialize state controller unlock condition: {}",
                                e
                            ))
                        })?,
                    ),
                    GovernorAddressUnlockCondition::KIND => UnlockConditionDto::GovernorAddress(
                        GovernorAddressUnlockConditionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize governor unlock condition: {}", e))
                        })?,
                    ),
                    ImmutableAliasAddressUnlockCondition::KIND => UnlockConditionDto::ImmutableAliasAddress(
                        ImmutableAliasAddressUnlockConditionDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!(
                                "cannot deserialize immutable alias address unlock condition: {}",
                                e
                            ))
                        })?,
                    ),
                    _ => return Err(serde::de::Error::custom("invalid unlock condition type")),
                },
            )
        }
    }

    impl Serialize for UnlockConditionDto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum UnlockConditionDto_<'a> {
                T1(&'a AddressUnlockConditionDto),
                T2(&'a StorageDepositReturnUnlockConditionDto),
                T3(&'a TimelockUnlockConditionDto),
                T4(&'a ExpirationUnlockConditionDto),
                T5(&'a StateControllerAddressUnlockConditionDto),
                T6(&'a GovernorAddressUnlockConditionDto),
                T7(&'a ImmutableAliasAddressUnlockConditionDto),
            }
            #[derive(Serialize)]
            struct TypedUnlockCondition<'a> {
                #[serde(flatten)]
                unlock_condition: UnlockConditionDto_<'a>,
            }
            let unlock_condition = match self {
                UnlockConditionDto::Address(o) => TypedUnlockCondition {
                    unlock_condition: UnlockConditionDto_::T1(o),
                },
                UnlockConditionDto::StorageDepositReturn(o) => TypedUnlockCondition {
                    unlock_condition: UnlockConditionDto_::T2(o),
                },
                UnlockConditionDto::Timelock(o) => TypedUnlockCondition {
                    unlock_condition: UnlockConditionDto_::T3(o),
                },
                UnlockConditionDto::Expiration(o) => TypedUnlockCondition {
                    unlock_condition: UnlockConditionDto_::T4(o),
                },
                UnlockConditionDto::StateControllerAddress(o) => TypedUnlockCondition {
                    unlock_condition: UnlockConditionDto_::T5(o),
                },
                UnlockConditionDto::GovernorAddress(o) => TypedUnlockCondition {
                    unlock_condition: UnlockConditionDto_::T6(o),
                },
                UnlockConditionDto::ImmutableAliasAddress(o) => TypedUnlockCondition {
                    unlock_condition: UnlockConditionDto_::T7(o),
                },
            };
            unlock_condition.serialize(serializer)
        }
    }

    impl From<&UnlockCondition> for UnlockConditionDto {
        fn from(value: &UnlockCondition) -> Self {
            match value {
                UnlockCondition::Address(v) => Self::Address(AddressUnlockConditionDto {
                    kind: AddressUnlockCondition::KIND,
                    address: v.address().into(),
                }),
                UnlockCondition::StorageDepositReturn(v) => {
                    Self::StorageDepositReturn(StorageDepositReturnUnlockConditionDto {
                        kind: StorageDepositReturnUnlockCondition::KIND,
                        return_address: AddressDto::from(v.return_address()),
                        amount: v.amount().to_string(),
                    })
                }
                UnlockCondition::Timelock(v) => Self::Timelock(TimelockUnlockConditionDto {
                    kind: TimelockUnlockCondition::KIND,
                    milestone_index: v.milestone_index(),
                    timestamp: v.timestamp(),
                }),
                UnlockCondition::Expiration(v) => Self::Expiration(ExpirationUnlockConditionDto {
                    kind: ExpirationUnlockCondition::KIND,
                    return_address: v.return_address().into(),
                    milestone_index: v.milestone_index(),
                    timestamp: v.timestamp(),
                }),
                UnlockCondition::StateControllerAddress(v) => {
                    Self::StateControllerAddress(StateControllerAddressUnlockConditionDto {
                        kind: StateControllerAddressUnlockCondition::KIND,
                        address: v.address().into(),
                    })
                }
                UnlockCondition::GovernorAddress(v) => Self::GovernorAddress(GovernorAddressUnlockConditionDto {
                    kind: GovernorAddressUnlockCondition::KIND,
                    address: v.address().into(),
                }),
                UnlockCondition::ImmutableAliasAddress(v) => {
                    Self::ImmutableAliasAddress(ImmutableAliasAddressUnlockConditionDto {
                        kind: ImmutableAliasAddressUnlockCondition::KIND,
                        address: AddressDto::Alias(v.address().into()),
                    })
                }
            }
        }
    }

    impl TryFrom<&UnlockConditionDto> for UnlockCondition {
        type Error = DtoError;

        fn try_from(value: &UnlockConditionDto) -> Result<Self, Self::Error> {
            Ok(match value {
                UnlockConditionDto::Address(v) => Self::Address(AddressUnlockCondition::new(
                    (&v.address)
                        .try_into()
                        .map_err(|_e| DtoError::InvalidField("AddressUnlockCondition"))?,
                )),
                UnlockConditionDto::StorageDepositReturn(v) => {
                    Self::StorageDepositReturn(StorageDepositReturnUnlockCondition::new(
                        Address::try_from(&v.return_address)?,
                        v.amount.parse::<u64>().map_err(|_| DtoError::InvalidField("amount"))?,
                    )?)
                }
                UnlockConditionDto::Timelock(v) => Self::Timelock(
                    TimelockUnlockCondition::new(v.milestone_index, v.timestamp)
                        .map_err(|_| DtoError::InvalidField("TimelockUnlockCondition"))?,
                ),
                UnlockConditionDto::Expiration(v) => Self::Expiration(
                    ExpirationUnlockCondition::new(
                        (&v.return_address)
                            .try_into()
                            .map_err(|_e| DtoError::InvalidField("ExpirationUnlockCondition"))?,
                        v.milestone_index,
                        v.timestamp,
                    )
                    .map_err(|_| DtoError::InvalidField("ExpirationUnlockCondition"))?,
                ),
                UnlockConditionDto::StateControllerAddress(v) => {
                    Self::StateControllerAddress(StateControllerAddressUnlockCondition::new(
                        (&v.address)
                            .try_into()
                            .map_err(|_e| DtoError::InvalidField("StateControllerAddressUnlockCondition"))?,
                    ))
                }
                UnlockConditionDto::GovernorAddress(v) => Self::GovernorAddress(GovernorAddressUnlockCondition::new(
                    (&v.address)
                        .try_into()
                        .map_err(|_e| DtoError::InvalidField("GovernorAddressUnlockCondition"))?,
                )),
                UnlockConditionDto::ImmutableAliasAddress(v) => {
                    let address: Address = (&v.address)
                        .try_into()
                        .map_err(|_e| DtoError::InvalidField("ImmutableAliasAddressUnlockCondition"))?;
                    // An ImmutableAliasAddressUnlockCondition must have an AliasAddress.
                    if let Address::Alias(alias_address) = &address {
                        Self::ImmutableAliasAddress(ImmutableAliasAddressUnlockCondition::new(*alias_address))
                    } else {
                        return Err(DtoError::InvalidField("ImmutableAliasAddressUnlockCondition"));
                    }
                }
            })
        }
    }

    impl UnlockConditionDto {
        /// Return the unlock condition kind of a `UnlockConditionDto`.
        pub fn kind(&self) -> u8 {
            match self {
                Self::Address(_) => AddressUnlockCondition::KIND,
                Self::StorageDepositReturn(_) => StorageDepositReturnUnlockCondition::KIND,
                Self::Timelock(_) => TimelockUnlockCondition::KIND,
                Self::Expiration(_) => ExpirationUnlockCondition::KIND,
                Self::StateControllerAddress(_) => StateControllerAddressUnlockCondition::KIND,
                Self::GovernorAddress(_) => GovernorAddressUnlockCondition::KIND,
                Self::ImmutableAliasAddress(_) => ImmutableAliasAddressUnlockCondition::KIND,
            }
        }
    }
}
