// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{validate_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        NativeToken, NativeTokens,
    },
    Error,
};

use bee_common::packable::{Packable as OldPackable, Read, Write};

///
pub struct ExtendedOutputBuilder {
    address: Address,
    amount: u64,
    native_tokens: Vec<NativeToken>,
    feature_blocks: Vec<FeatureBlock>,
}

impl ExtendedOutputBuilder {
    ///
    #[inline(always)]
    pub fn new(address: Address, amount: u64) -> Self {
        Self {
            address,
            amount,
            native_tokens: Vec::new(),
            feature_blocks: Vec::new(),
        }
    }

    ///
    #[inline(always)]
    pub fn add_native_token(mut self, native_token: NativeToken) -> Self {
        self.native_tokens.push(native_token);
        self
    }

    ///
    #[inline(always)]
    pub fn with_native_tokens(mut self, native_tokens: impl IntoIterator<Item = NativeToken>) -> Self {
        self.native_tokens = native_tokens.into_iter().collect();
        self
    }

    ///
    #[inline(always)]
    pub fn add_feature_block(mut self, feature_block: FeatureBlock) -> Self {
        self.feature_blocks.push(feature_block);
        self
    }

    ///
    #[inline(always)]
    pub fn with_feature_blocks(mut self, feature_blocks: impl IntoIterator<Item = FeatureBlock>) -> Self {
        self.feature_blocks = feature_blocks.into_iter().collect();
        self
    }

    ///
    pub fn finish(self) -> Result<ExtendedOutput, Error> {
        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        validate_allowed_feature_blocks(&feature_blocks, ExtendedOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(ExtendedOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            feature_blocks,
        })
    }
}

/// Describes an extended output with optional features.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtendedOutput {
    // Deposit address of the output.
    address: Address,
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    feature_blocks: FeatureBlocks,
}

impl ExtendedOutput {
    /// The [`Output`](crate::output::Output) kind of an [`ExtendedOutput`].
    pub const KIND: u8 = 3;

    /// The set of allowed [`FeatureBlock`]s for an [`ExtendedOutput`].
    const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER
        .union(FeatureBlockFlags::DUST_DEPOSIT_RETURN)
        .union(FeatureBlockFlags::TIMELOCK_MILESTONE_INDEX)
        .union(FeatureBlockFlags::TIMELOCK_UNIX)
        .union(FeatureBlockFlags::EXPIRATION_MILESTONE_INDEX)
        .union(FeatureBlockFlags::EXPIRATION_UNIX)
        .union(FeatureBlockFlags::METADATA)
        .union(FeatureBlockFlags::INDEXATION);

    /// Creates a new [`ExtendedOutput`].
    #[inline(always)]
    pub fn new(address: Address, amount: u64) -> Self {
        // SAFETY: this can't fail as this is a default builder.
        ExtendedOutputBuilder::new(address, amount).finish().unwrap()
    }

    /// Creates a new [`ExtendedOutputBuilder`].
    #[inline(always)]
    pub fn build(address: Address, amount: u64) -> ExtendedOutputBuilder {
        ExtendedOutputBuilder::new(address, amount)
    }

    ///
    #[inline(always)]
    pub fn address(&self) -> &Address {
        &self.address
    }

    ///
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    ///
    #[inline(always)]
    pub fn native_tokens(&self) -> &[NativeToken] {
        &self.native_tokens
    }

    ///
    #[inline(always)]
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl OldPackable for ExtendedOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.address.packed_len()
            + self.amount.packed_len()
            + self.native_tokens.packed_len()
            + self.feature_blocks.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        self.amount.pack(writer)?;
        self.native_tokens.pack(writer)?;
        self.feature_blocks.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let address = Address::unpack_inner::<R, CHECK>(reader)?;
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let native_tokens = NativeTokens::unpack_inner::<R, CHECK>(reader)?;
        let feature_blocks = FeatureBlocks::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_allowed_feature_blocks(&feature_blocks, ExtendedOutput::ALLOWED_FEATURE_BLOCKS)?;
        }

        Ok(Self {
            address,
            amount,
            native_tokens,
            feature_blocks,
        })
    }
}
