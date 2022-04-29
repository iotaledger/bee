// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use derive_more::{AsRef, Deref, From};

use crate::{output::NftId, Error};

/// An NFT address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, From, AsRef, Deref, packable::Packable)]
#[as_ref(forward)]
pub struct NftAddress(NftId);

#[allow(clippy::len_without_is_empty)]
impl NftAddress {
    /// The [`Address`](crate::address::Address) kind of an NFT address.
    pub const KIND: u8 = 16;
    /// The length of a [`NftAddress`].
    pub const LENGTH: usize = NftId::LENGTH;

    /// Creates a new [`NftAddress`].
    #[inline(always)]
    pub fn new(id: NftId) -> Self {
        Self::from(id)
    }

    /// Returns the [`NftId`] of an [`NftAddress`].
    #[inline(always)]
    pub fn nft_id(&self) -> &NftId {
        &self.0
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(NftAddress);

impl FromStr for NftAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NftAddress::from(NftId::from_str(s)?))
    }
}

impl core::fmt::Display for NftAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for NftAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "NftAddress({})", self)
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    /// Describes an NFT address.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct NftAddressDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "nftId")]
        pub nft_id: String,
    }

    impl From<&NftAddress> for NftAddressDto {
        fn from(value: &NftAddress) -> Self {
            Self {
                kind: NftAddress::KIND,
                nft_id: value.to_string(),
            }
        }
    }

    impl TryFrom<&NftAddressDto> for NftAddress {
        type Error = DtoError;

        fn try_from(value: &NftAddressDto) -> Result<Self, Self::Error> {
            value
                .nft_id
                .parse::<NftAddress>()
                .map_err(|_| DtoError::InvalidField("NFT address"))
        }
    }
}
