// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{validate_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{
            validate_allowed_unlock_conditions, UnlockCondition, UnlockConditionFlags, UnlockConditions,
        },
        NativeToken, NativeTokens,
    },
    Error,
};

use packable::Packable;

///
#[must_use]
pub struct ExtendedOutputBuilder {
    address: Address,
    amount: u64,
    native_tokens: Vec<NativeToken>,
    unlock_conditions: Vec<UnlockCondition>,
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
            unlock_conditions: Vec::new(),
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
    pub fn add_unlock_condition(mut self, unlock_condition: UnlockCondition) -> Self {
        self.unlock_conditions.push(unlock_condition);
        self
    }

    ///
    #[inline(always)]
    pub fn with_unlock_conditions(mut self, unlock_conditions: impl IntoIterator<Item = UnlockCondition>) -> Self {
        self.unlock_conditions = unlock_conditions.into_iter().collect();
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
        let unlock_conditions = UnlockConditions::new(self.unlock_conditions)?;

        validate_allowed_unlock_conditions(&unlock_conditions, ExtendedOutput::ALLOWED_UNLOCK_CONDITIONS)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        validate_allowed_feature_blocks(&feature_blocks, ExtendedOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(ExtendedOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            unlock_conditions,
            feature_blocks,
        })
    }
}

/// Describes an extended output with optional features.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct ExtendedOutput {
    // Deposit address of the output.
    address: Address,
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    #[packable(verify_with = Self::validate_unlock_conditions)]
    unlock_conditions: UnlockConditions,
    #[packable(verify_with = Self::validate_feature_blocks)]
    feature_blocks: FeatureBlocks,
}

impl ExtendedOutput {
    /// The [`Output`](crate::output::Output) kind of an [`ExtendedOutput`].
    pub const KIND: u8 = 3;

    /// The set of allowed [`UnlockCondition`]s for an [`ExtendedOutput`].
    const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::DUST_DEPOSIT_RETURN
        .union(UnlockConditionFlags::TIMELOCK)
        .union(UnlockConditionFlags::EXPIRATION);
    /// The set of allowed [`FeatureBlock`]s for an [`ExtendedOutput`].
    const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER
        .union(FeatureBlockFlags::METADATA)
        .union(FeatureBlockFlags::TAG);

    fn validate_unlock_conditions<const VERIFY: bool>(unlock_conditions: &UnlockConditions) -> Result<(), Error> {
        if VERIFY {
            validate_allowed_unlock_conditions(unlock_conditions, ExtendedOutput::ALLOWED_UNLOCK_CONDITIONS)
        } else {
            Ok(())
        }
    }

    fn validate_feature_blocks<const VERIFY: bool>(blocks: &FeatureBlocks) -> Result<(), Error> {
        if VERIFY {
            validate_allowed_feature_blocks(blocks, ExtendedOutput::ALLOWED_FEATURE_BLOCKS)
        } else {
            Ok(())
        }
    }

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
    pub fn unlock_conditions(&self) -> &[UnlockCondition] {
        &self.unlock_conditions
    }

    ///
    #[inline(always)]
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}
