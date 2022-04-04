// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

use packable::Packable;

use crate::{
    address::Address,
    output::{
        feature_block::{verify_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{verify_allowed_unlock_conditions, UnlockCondition, UnlockConditionFlags, UnlockConditions},
        ByteCost, ByteCostConfig, NativeToken, NativeTokens, Output, OutputAmount, OutputBuilderAmount, OutputId,
    },
    semantic::{ConflictReason, ValidationContext},
    unlock_block::UnlockBlock,
    Error,
};

///
#[must_use]
pub struct BasicOutputBuilder {
    amount: OutputBuilderAmount,
    native_tokens: Vec<NativeToken>,
    unlock_conditions: Vec<UnlockCondition>,
    feature_blocks: Vec<FeatureBlock>,
}

impl BasicOutputBuilder {
    /// Creates a [`BasicOutputBuilder`] with a provided amount.
    #[inline(always)]
    pub fn new_with_amount(amount: u64) -> Result<Self, Error> {
        Ok(Self {
            amount: OutputBuilderAmount::Amount(amount.try_into().map_err(Error::InvalidOutputAmount)?),
            native_tokens: Vec::new(),
            unlock_conditions: Vec::new(),
            feature_blocks: Vec::new(),
        })
    }

    /// Creates an [`BasicOutputBuilder`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn new_with_minimum_storage_deposit(byte_cost_config: ByteCostConfig) -> Result<Self, Error> {
        Ok(Self {
            amount: OutputBuilderAmount::MinimumStorageDeposit(byte_cost_config),
            native_tokens: Vec::new(),
            unlock_conditions: Vec::new(),
            feature_blocks: Vec::new(),
        })
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
    pub fn finish(self) -> Result<BasicOutput, Error> {
        let unlock_conditions = UnlockConditions::new(self.unlock_conditions)?;

        verify_unlock_conditions::<true>(&unlock_conditions)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        verify_feature_blocks::<true>(&feature_blocks)?;

        let mut output = BasicOutput {
            amount: 1u64.try_into().map_err(Error::InvalidOutputAmount)?,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            unlock_conditions,
            feature_blocks,
        };

        output.amount = match self.amount {
            OutputBuilderAmount::Amount(amount) => amount,
            OutputBuilderAmount::MinimumStorageDeposit(byte_cost_config) => Output::Basic(output.clone())
                .byte_cost(&byte_cost_config)
                .try_into()
                .map_err(Error::InvalidOutputAmount)?,
        };

        Ok(output)
    }
}

/// Describes a basic output with optional features.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
pub struct BasicOutput {
    // Amount of IOTA tokens held by the output.
    #[packable(unpack_error_with = Error::InvalidOutputAmount)]
    amount: OutputAmount,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    #[packable(verify_with = verify_unlock_conditions)]
    unlock_conditions: UnlockConditions,
    #[packable(verify_with = verify_feature_blocks)]
    feature_blocks: FeatureBlocks,
}

impl BasicOutput {
    /// The [`Output`](crate::output::Output) kind of an [`BasicOutput`].
    pub const KIND: u8 = 3;

    /// The set of allowed [`UnlockCondition`]s for an [`BasicOutput`].
    const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::ADDRESS
        .union(UnlockConditionFlags::STORAGE_DEPOSIT_RETURN)
        .union(UnlockConditionFlags::TIMELOCK)
        .union(UnlockConditionFlags::EXPIRATION);
    /// The set of allowed [`FeatureBlock`]s for an [`BasicOutput`].
    pub const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER
        .union(FeatureBlockFlags::METADATA)
        .union(FeatureBlockFlags::TAG);

    /// Creates a new [`BasicOutput`] with a provided amount.
    #[inline(always)]
    pub fn new_with_amount(amount: u64) -> Result<Self, Error> {
        BasicOutputBuilder::new_with_amount(amount)?.finish()
    }

    /// Creates a new [`BasicOutput`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn new_with_minimum_storage_deposit(byte_cost_config: ByteCostConfig) -> Result<Self, Error> {
        BasicOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config)?.finish()
    }

    /// Creates a new [`BasicOutputBuilder`] with a provided amount.
    #[inline(always)]
    pub fn build_with_amount(amount: u64) -> Result<BasicOutputBuilder, Error> {
        BasicOutputBuilder::new_with_amount(amount)
    }

    /// Creates a new [`BasicOutputBuilder`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn build_with_minimum_storage_deposit(byte_cost_config: ByteCostConfig) -> Result<BasicOutputBuilder, Error> {
        BasicOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config)
    }

    ///
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }

    ///
    #[inline(always)]
    pub fn native_tokens(&self) -> &NativeTokens {
        &self.native_tokens
    }

    ///
    #[inline(always)]
    pub fn unlock_conditions(&self) -> &UnlockConditions {
        &self.unlock_conditions
    }

    ///
    #[inline(always)]
    pub fn feature_blocks(&self) -> &FeatureBlocks {
        &self.feature_blocks
    }

    ///
    #[inline(always)]
    pub fn address(&self) -> &Address {
        // An BasicOutput must have an AddressUnlockCondition.
        self.unlock_conditions
            .address()
            .map(|unlock_condition| unlock_condition.address())
            .unwrap()
    }

    ///
    pub fn unlock(
        &self,
        _output_id: &OutputId,
        unlock_block: &UnlockBlock,
        inputs: &[(OutputId, &Output)],
        context: &mut ValidationContext,
    ) -> Result<(), ConflictReason> {
        let locked_address = self.unlock_conditions().locked_address(
            self.address(),
            context.milestone_index,
            context.milestone_timestamp,
        );

        locked_address.unlock(unlock_block, inputs, context)
    }
}

fn verify_unlock_conditions<const VERIFY: bool>(unlock_conditions: &UnlockConditions) -> Result<(), Error> {
    if VERIFY {
        if unlock_conditions.address().is_none() {
            Err(Error::MissingAddressUnlockCondition)
        } else {
            verify_allowed_unlock_conditions(unlock_conditions, BasicOutput::ALLOWED_UNLOCK_CONDITIONS)
        }
    } else {
        Ok(())
    }
}

fn verify_feature_blocks<const VERIFY: bool>(blocks: &FeatureBlocks) -> Result<(), Error> {
    if VERIFY {
        verify_allowed_feature_blocks(blocks, BasicOutput::ALLOWED_FEATURE_BLOCKS)
    } else {
        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        error::dto::DtoError,
        output::{
            feature_block::dto::FeatureBlockDto, native_token::dto::NativeTokenDto,
            unlock_condition::dto::UnlockConditionDto,
        },
    };

    /// Describes a basic output.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct BasicOutputDto {
        #[serde(rename = "type")]
        pub kind: u8,
        // Amount of IOTA tokens held by the output.
        pub amount: String,
        // Native tokens held by the output.
        #[serde(rename = "nativeTokens")]
        pub native_tokens: Vec<NativeTokenDto>,
        #[serde(rename = "unlockConditions")]
        pub unlock_conditions: Vec<UnlockConditionDto>,
        #[serde(rename = "featureBlocks")]
        pub feature_blocks: Vec<FeatureBlockDto>,
    }

    impl From<&BasicOutput> for BasicOutputDto {
        fn from(value: &BasicOutput) -> Self {
            Self {
                kind: BasicOutput::KIND,
                amount: value.amount().to_string(),
                native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
                unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
                feature_blocks: value.feature_blocks().iter().map(Into::into).collect::<_>(),
            }
        }
    }

    impl TryFrom<&BasicOutputDto> for BasicOutput {
        type Error = DtoError;

        fn try_from(value: &BasicOutputDto) -> Result<Self, Self::Error> {
            let mut builder = BasicOutputBuilder::new_with_amount(
                value
                    .amount
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("amount"))?,
            )?;
            for t in &value.native_tokens {
                builder = builder.add_native_token(t.try_into()?);
            }
            for b in &value.unlock_conditions {
                builder = builder.add_unlock_condition(b.try_into()?);
            }
            for b in &value.feature_blocks {
                builder = builder.add_feature_block(b.try_into()?);
            }
            Ok(builder.finish()?)
        }
    }
}
