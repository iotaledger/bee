// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::AliasId, util::hex_decode, Error};

use derive_more::{AsRef, Deref, From};

use core::str::FromStr;

/// An alias address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, From, AsRef, Deref, bee_packable::Packable)]
#[as_ref(forward)]
pub struct AliasAddress(AliasId);

#[allow(clippy::len_without_is_empty)]
impl AliasAddress {
    /// The [`Address`](crate::address::Address) kind of an [`AliasAddress`].
    pub const KIND: u8 = 8;
    /// The length of an [`AliasAddress`].
    pub const LENGTH: usize = 20;

    /// Creates a new [`AliasAddress`].
    #[inline(always)]
    pub fn new(id: AliasId) -> Self {
        Self::from(id)
    }

    /// Returns the [`AliasId`] of an [`AliasAddress`].
    #[inline(always)]
    pub fn id(&self) -> &AliasId {
        &self.0
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(AliasAddress);

impl FromStr for AliasAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AliasAddress::from(AliasId::from(hex_decode(s)?)))
    }
}

impl core::fmt::Display for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "AliasAddress({})", self)
    }
}
