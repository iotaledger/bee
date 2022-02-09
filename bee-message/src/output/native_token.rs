// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::TokenId, Error};

use derive_more::Deref;
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable};
use primitive_types::U256;

use alloc::vec::Vec;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct NativeToken {
    // Identifier of the native token.
    token_id: TokenId,
    // Amount of native tokens.
    #[packable(verify_with = verify_amount)]
    amount: U256,
}

impl NativeToken {
    /// Creates a new [`NativeToken`].
    #[inline(always)]
    pub fn new(token_id: TokenId, amount: U256) -> Result<Self, Error> {
        verify_amount::<true>(&amount)?;

        Ok(Self { token_id, amount })
    }

    /// Returns the token ID of the [`NativeToken`].
    #[inline(always)]
    pub fn token_id(&self) -> &TokenId {
        &self.token_id
    }

    /// Returns the amount of the [`NativeToken`].
    #[inline(always)]
    pub fn amount(&self) -> &U256 {
        &self.amount
    }
}

#[inline]
fn verify_amount<const VERIFY: bool>(amount: &U256) -> Result<(), Error> {
    if VERIFY && amount.is_zero() {
        Err(Error::NativeTokensNullAmount)
    } else {
        Ok(())
    }
}

pub(crate) type NativeTokenCount = BoundedU8<0, { NativeTokens::COUNT_MAX }>;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deref, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidNativeTokenCount(p.into())))]
pub struct NativeTokens(
    #[packable(verify_with = verify_unique_sorted)] BoxedSlicePrefix<NativeToken, NativeTokenCount>,
);

impl TryFrom<Vec<NativeToken>> for NativeTokens {
    type Error = Error;

    #[inline(always)]
    fn try_from(native_tokens: Vec<NativeToken>) -> Result<Self, Self::Error> {
        Self::new(native_tokens)
    }
}

impl NativeTokens {
    /// Maximum number of different native tokens that can be referenced in one transaction.
    pub const COUNT_MAX: u8 = 64;

    /// Creates a new [`NativeTokens`].
    pub fn new(native_tokens: Vec<NativeToken>) -> Result<Self, Error> {
        let mut native_tokens =
            BoxedSlicePrefix::<NativeToken, NativeTokenCount>::try_from(native_tokens.into_boxed_slice())
                .map_err(Error::InvalidNativeTokenCount)?;

        native_tokens.sort_by(|a, b| a.token_id().cmp(b.token_id()));
        // Sort is obviously fine now but uniqueness still needs to be checked.
        verify_unique_sorted::<true>(&native_tokens)?;

        Ok(Self(native_tokens))
    }
}

#[inline]
fn verify_unique_sorted<const VERIFY: bool>(native_tokens: &[NativeToken]) -> Result<(), Error> {
    if VERIFY && !is_unique_sorted(native_tokens.iter().map(NativeToken::token_id)) {
        Err(Error::NativeTokensNotUniqueSorted)
    } else {
        Ok(())
    }
}
