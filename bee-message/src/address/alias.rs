// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{hex::hex_encode_prefix, output::AliasId, Error};

use derive_more::{AsRef, Deref, From};

use core::str::FromStr;

/// An alias address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, From, AsRef, Deref, packable::Packable)]
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
    pub fn alias_id(&self) -> &AliasId {
        &self.0
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(AliasAddress);

impl FromStr for AliasAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AliasAddress::new(AliasId::from_str(s)?))
    }
}

impl core::fmt::Display for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex_encode_prefix(self.0))
    }
}

impl core::fmt::Debug for AliasAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "AliasAddress({})", self)
    }
}
