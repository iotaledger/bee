// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};
use primitive_types::U256;

use crate::error::Error;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimpleTokenScheme {
    // Amount of tokens minted by a foundry.
    minted_tokens: U256,
    // Amount of tokens melted by a foundry.
    melted_tokens: U256,
    // Maximum supply of tokens controlled by a foundry.
    maximum_supply: U256,
}

impl SimpleTokenScheme {
    /// The [`TokenScheme`](crate::output::TokenScheme) kind of a [`SimpleTokenScheme`].
    pub const KIND: u8 = 0;

    /// Creates a new [`SimpleTokenScheme`].
    #[inline(always)]
    pub fn new(minted_tokens: U256, melted_tokens: U256, maximum_supply: U256) -> Result<Self, Error> {
        verify_supply(&minted_tokens, &melted_tokens, &maximum_supply)?;

        Ok(Self {
            minted_tokens,
            melted_tokens,
            maximum_supply,
        })
    }

    /// Returns the number of minted tokens of the [`SimpleTokenScheme`].
    #[inline(always)]
    pub fn minted_tokens(&self) -> &U256 {
        &self.minted_tokens
    }

    /// Returns the number of melted tokens of the [`SimpleTokenScheme`].
    #[inline(always)]
    pub fn melted_tokens(&self) -> &U256 {
        &self.melted_tokens
    }

    /// Returns the maximum supply of the [`SimpleTokenScheme`].
    #[inline(always)]
    pub fn maximum_supply(&self) -> &U256 {
        &self.maximum_supply
    }

    /// Returns the circulating supply of the [`SimpleTokenScheme`].
    #[inline(always)]
    pub fn circulating_supply(&self) -> U256 {
        self.minted_tokens - self.melted_tokens
    }
}

impl Packable for SimpleTokenScheme {
    type UnpackError = Error;
    type UnpackVisitor = ();

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.minted_tokens.pack(packer)?;
        self.melted_tokens.pack(packer)?;
        self.maximum_supply.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &mut Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let minted_tokens = U256::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;
        let melted_tokens = U256::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;
        let maximum_supply = U256::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;

        if VERIFY {
            verify_supply(&minted_tokens, &melted_tokens, &maximum_supply).map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            minted_tokens,
            melted_tokens,
            maximum_supply,
        })
    }
}

#[inline]
fn verify_supply(minted_tokens: &U256, melted_tokens: &U256, maximum_supply: &U256) -> Result<(), Error> {
    if maximum_supply.is_zero() || melted_tokens > minted_tokens || minted_tokens - melted_tokens > *maximum_supply {
        return Err(Error::InvalidFoundryOutputSupply {
            minted: *minted_tokens,
            melted: *melted_tokens,
            max: *maximum_supply,
        });
    }

    Ok(())
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{dto::U256Dto, error::dto::DtoError};

    /// Describes a foundry output that is controlled by an alias.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct SimpleTokenSchemeDto {
        #[serde(rename = "type")]
        pub kind: u8,
        // Amount of tokens minted by a foundry.
        #[serde(rename = "mintedTokens")]
        pub minted_tokens: U256Dto,
        // Amount of tokens melted by a foundry.
        #[serde(rename = "meltedTokens")]
        pub melted_tokens: U256Dto,
        // Maximum supply of tokens controlled by a foundry.
        #[serde(rename = "maximumSupply")]
        pub maximum_supply: U256Dto,
    }

    impl From<&SimpleTokenScheme> for SimpleTokenSchemeDto {
        fn from(value: &SimpleTokenScheme) -> Self {
            Self {
                kind: SimpleTokenScheme::KIND,
                minted_tokens: value.minted_tokens().into(),
                melted_tokens: value.melted_tokens().into(),
                maximum_supply: value.maximum_supply().into(),
            }
        }
    }

    impl TryFrom<&SimpleTokenSchemeDto> for SimpleTokenScheme {
        type Error = DtoError;

        fn try_from(value: &SimpleTokenSchemeDto) -> Result<Self, Self::Error> {
            Self::new(
                U256::try_from(&value.minted_tokens).map_err(|_| DtoError::InvalidField("mintedTokens"))?,
                U256::try_from(&value.melted_tokens).map_err(|_| DtoError::InvalidField("meltedTokens"))?,
                U256::try_from(&value.maximum_supply).map_err(|_| DtoError::InvalidField("maximumSupply"))?,
            )
            .map_err(DtoError::Block)
        }
    }
}
