// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use derive_more::{AsRef, Deref, From};

use crate::{output::AliasId, Error};

/// An alias address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, From, AsRef, Deref, packable::Packable)]
#[as_ref(forward)]
pub struct AliasAddress(AliasId);

#[allow(clippy::len_without_is_empty)]
impl AliasAddress {
    /// The [`Address`](crate::address::Address) kind of an [`AliasAddress`].
    pub const KIND: u8 = 8;
    /// The length of an [`AliasAddress`].
    pub const LENGTH: usize = AliasId::LENGTH;

    /// Creates a new [`AliasAddress`].
    #[inline(always)]
    pub fn new(id: AliasId) -> Self {
        Self::from(id)
    }

    /// Returns the [`AliasId`] of an [`AliasAddress`].
    #[inline(always)]
    pub fn alias_id(&self) -> &AliasId {
        &self.0
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(AliasAddress);

impl FromStr for AliasAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AliasAddress::new(AliasId::from_str(s)?))
    }
}

impl core::fmt::Display for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "AliasAddress({})", self)
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    /// Describes an alias address.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct AliasAddressDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "aliasId")]
        pub alias_id: String,
    }

    impl From<&AliasAddress> for AliasAddressDto {
        fn from(value: &AliasAddress) -> Self {
            Self {
                kind: AliasAddress::KIND,
                alias_id: value.to_string(),
            }
        }
    }

    impl TryFrom<&AliasAddressDto> for AliasAddress {
        type Error = DtoError;

        fn try_from(value: &AliasAddressDto) -> Result<Self, Self::Error> {
            value
                .alias_id
                .parse::<AliasAddress>()
                .map_err(|_| DtoError::InvalidField("alias address"))
        }
    }
}
