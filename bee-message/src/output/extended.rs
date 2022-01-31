// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{verify_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{
            verify_allowed_unlock_conditions, AddressUnlockCondition, UnlockCondition, UnlockConditionFlags,
            UnlockConditions,
        },
        NativeToken, NativeTokens,
    },
    Error,
};

use packable::Packable;

use alloc::vec::Vec;

///
#[must_use]
pub struct ExtendedOutputBuilder {
    amount: u64,
    native_tokens: Vec<NativeToken>,
    unlock_conditions: Vec<UnlockCondition>,
    feature_blocks: Vec<FeatureBlock>,
}

impl ExtendedOutputBuilder {
    ///
    #[inline(always)]
    pub fn new(amount: u64) -> Self {
        Self {
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

        verify_unlock_conditions::<true>(&unlock_conditions)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        verify_feature_blocks::<true>(&feature_blocks)?;

        Ok(ExtendedOutput {
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
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    #[packable(verify_with = verify_unlock_conditions)]
    unlock_conditions: UnlockConditions,
    #[packable(verify_with = verify_feature_blocks)]
    feature_blocks: FeatureBlocks,
}

impl ExtendedOutput {
    /// The [`Output`](crate::output::Output) kind of an [`ExtendedOutput`].
    pub const KIND: u8 = 3;

    /// The set of allowed [`UnlockCondition`]s for an [`ExtendedOutput`].
    const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::ADDRESS
        .union(UnlockConditionFlags::DUST_DEPOSIT_RETURN)
        .union(UnlockConditionFlags::TIMELOCK)
        .union(UnlockConditionFlags::EXPIRATION);
    /// The set of allowed [`FeatureBlock`]s for an [`ExtendedOutput`].
    pub const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER
        .union(FeatureBlockFlags::METADATA)
        .union(FeatureBlockFlags::TAG);

    /// Creates a new [`ExtendedOutput`].
    #[inline(always)]
    pub fn new(amount: u64) -> Self {
        // SAFETY: this can't fail as this is a default builder.
        ExtendedOutputBuilder::new(amount).finish().unwrap()
    }

    /// Creates a new [`ExtendedOutputBuilder`].
    #[inline(always)]
    pub fn build(amount: u64) -> ExtendedOutputBuilder {
        ExtendedOutputBuilder::new(amount)
    }

    ///
    #[inline(always)]
    pub fn address(&self) -> &Address {
        // An ExtendedOutput must have a AddressUnlockCondition.
        if let UnlockCondition::Address(address) = self.unlock_conditions.get(AddressUnlockCondition::KIND).unwrap() {
            address.address()
        } else {
            unreachable!();
        }
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

fn verify_unlock_conditions<const VERIFY: bool>(unlock_conditions: &UnlockConditions) -> Result<(), Error> {
    if VERIFY {
        if unlock_conditions.get(AddressUnlockCondition::KIND).is_none() {
            Err(Error::MissingAddressUnlockCondition)
        } else {
            verify_allowed_unlock_conditions(unlock_conditions, ExtendedOutput::ALLOWED_UNLOCK_CONDITIONS)
        }
    } else {
        Ok(())
    }
}

fn verify_feature_blocks<const VERIFY: bool>(blocks: &FeatureBlocks) -> Result<(), Error> {
    if VERIFY {
        verify_allowed_feature_blocks(blocks, ExtendedOutput::ALLOWED_FEATURE_BLOCKS)
    } else {
        Ok(())
    }
}
