// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{packer::SlicePacker, Packable};

use crate::output::FoundryId;

impl_id!(pub TokenTag, 12, "TODO.");

#[cfg(feature = "serde")]
string_serde_impl!(TokenTag);

impl_id!(pub TokenId, 50, "TODO.");

#[cfg(feature = "serde")]
string_serde_impl!(TokenId);

impl TokenId {
    /// Builds a new [`TokenId`] from its components.
    pub fn build(foundry_id: &FoundryId, token_tag: &TokenTag) -> Self {
        let mut bytes = [0u8; TokenId::LENGTH];
        let mut packer = SlicePacker::new(&mut bytes);

        // PANIC: packing to an array of the correct length can't fail.
        foundry_id.pack(&mut packer).unwrap();
        token_tag.pack(&mut packer).unwrap();

        TokenId::new(bytes)
    }

    /// Returns the [`FoundryId`] of the [`TokenId`].
    pub fn foundry_id(&self) -> FoundryId {
        // PANIC: the lengths are known.
        FoundryId::new(self.0[0..FoundryId::LENGTH].try_into().unwrap())
    }

    /// Returns the [`TokenTag`] of the [`TokenId`].
    pub fn token_tag(&self) -> TokenTag {
        // PANIC: the lengths are known.
        TokenTag::new(self.0[FoundryId::LENGTH..].try_into().unwrap())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TokenTagDto(pub String);

    impl From<&TokenTag> for TokenTagDto {
        fn from(value: &TokenTag) -> Self {
            Self(value.to_string())
        }
    }

    impl TryFrom<&TokenTagDto> for TokenTag {
        type Error = DtoError;

        fn try_from(value: &TokenTagDto) -> Result<Self, Self::Error> {
            value
                .0
                .parse::<TokenTag>()
                .map_err(|_| DtoError::InvalidField("TokenTag"))
        }
    }

    /// Describes a token id.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TokenIdDto(pub String);

    impl From<&TokenId> for TokenIdDto {
        fn from(value: &TokenId) -> Self {
            Self(prefix_hex::encode(**value))
        }
    }

    impl TryFrom<&TokenIdDto> for TokenId {
        type Error = DtoError;

        fn try_from(value: &TokenIdDto) -> Result<Self, Self::Error> {
            let token_id: [u8; TokenId::LENGTH] =
                prefix_hex::decode(&value.0).map_err(|_e| DtoError::InvalidField("tokenId"))?;
            Ok(TokenId::new(token_id))
        }
    }
}
