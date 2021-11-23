// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::TokenId, Error};

use bee_common::packable::{Packable, Read, Write};

use primitive_types::U256;

use core::ops::Deref;

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct NativeToken {
    // Identifier of the native token.
    token_id: TokenId,
    // Amount of native tokens.
    amount: U256,
}

impl NativeToken {
    /// Creates a new `NativeToken`.
    pub fn new(token_id: TokenId, amount: U256) -> Self {
        Self { token_id, amount }
    }

    /// Returns the token ID of the `NativeToken`.
    pub fn token_id(&self) -> &TokenId {
        &self.token_id
    }

    /// Returns the amount of the `NativeToken`.
    pub fn amount(&self) -> &U256 {
        &self.amount
    }
}

impl Packable for NativeToken {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.token_id.packed_len() + 32
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.token_id.pack(writer)?;
        // SAFETY: Reinterpreting a [u64; 4] as a [u8; 32] is fine since they have the same size.
        writer.write_all(&unsafe { std::mem::transmute::<[u64; 4], [u8; 32]>(self.amount.0) })?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let token_id = TokenId::unpack_inner::<R, CHECK>(reader)?;
        let amount = U256::from_little_endian(&<[u8; 32]>::unpack_inner::<R, CHECK>(reader)?);

        Ok(Self::new(token_id, amount))
    }
}

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct NativeTokens(Box<[NativeToken]>);

impl TryFrom<Vec<NativeToken>> for NativeTokens {
    type Error = Error;

    fn try_from(native_tokens: Vec<NativeToken>) -> Result<Self, Self::Error> {
        validate_count(native_tokens.len())?;

        Ok(Self(native_tokens.into_boxed_slice()))
    }
}

impl NativeTokens {
    ///
    pub const COUNT_MAX: usize = 256;

    /// Creates a new `NativeTokens`.
    pub fn new(native_tokens: Vec<NativeToken>) -> Result<Self, Error> {
        Self::try_from(native_tokens)
    }
}

impl Deref for NativeTokens {
    type Target = [NativeToken];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Packable for NativeTokens {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len() + self.0.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.0.len() as u16).pack(writer)?;
        for native_token in self.0.iter() {
            native_token.pack(writer)?
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let native_tokens_count = u16::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK {
            validate_count(native_tokens_count)?;
        }

        let mut native_tokens = Vec::with_capacity(native_tokens_count);
        for _ in 0..native_tokens_count {
            native_tokens.push(NativeToken::unpack_inner::<R, CHECK>(reader)?);
        }

        Self::new(native_tokens)
    }
}

#[inline]
fn validate_count(native_tokens_count: usize) -> Result<(), Error> {
    if native_tokens_count > NativeTokens::COUNT_MAX {
        return Err(Error::InvalidNativeTokenCount(native_tokens_count));
    }

    Ok(())
}
