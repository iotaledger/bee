// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;
use core::cmp::Ordering;

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use crate::{
    address::{Address, AliasAddress},
    output::{
        feature_block::{verify_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{verify_allowed_unlock_conditions, UnlockCondition, UnlockConditionFlags, UnlockConditions},
        ByteCost, ByteCostConfig, ChainId, FoundryId, NativeToken, NativeTokens, Output, OutputAmount,
        OutputBuilderAmount, OutputId, StateTransitionError, StateTransitionVerifier, TokenId, TokenScheme, TokenTag,
    },
    semantic::{ConflictReason, ValidationContext},
    unlock_block::UnlockBlock,
    Error,
};

///
#[must_use]
pub struct FoundryOutputBuilder {
    amount: OutputBuilderAmount,
    native_tokens: Vec<NativeToken>,
    serial_number: u32,
    token_tag: TokenTag,
    token_scheme: TokenScheme,
    unlock_conditions: Vec<UnlockCondition>,
    feature_blocks: Vec<FeatureBlock>,
    immutable_feature_blocks: Vec<FeatureBlock>,
}

impl FoundryOutputBuilder {
    /// Creates a [`FoundryOutputBuilder`] with a provided amount.
    pub fn new_with_amount(
        amount: u64,
        serial_number: u32,
        token_tag: TokenTag,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        Self::new(
            OutputBuilderAmount::Amount(amount.try_into().map_err(Error::InvalidOutputAmount)?),
            serial_number,
            token_tag,
            token_scheme,
        )
    }

    /// Creates a [`FoundryOutputBuilder`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    pub fn new_with_minimum_storage_deposit(
        byte_cost_config: ByteCostConfig,
        serial_number: u32,
        token_tag: TokenTag,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        Self::new(
            OutputBuilderAmount::MinimumStorageDeposit(byte_cost_config),
            serial_number,
            token_tag,
            token_scheme,
        )
    }

    fn new(
        amount: OutputBuilderAmount,
        serial_number: u32,
        token_tag: TokenTag,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        Ok(Self {
            amount,
            native_tokens: Vec::new(),
            serial_number,
            token_tag,
            token_scheme,
            unlock_conditions: Vec::new(),
            feature_blocks: Vec::new(),
            immutable_feature_blocks: Vec::new(),
        })
    }

    /// Sets the amount to the provided value.
    #[inline(always)]
    pub fn with_amount(mut self, amount: u64) -> Result<Self, Error> {
        self.amount = OutputBuilderAmount::Amount(amount.try_into().map_err(Error::InvalidOutputAmount)?);
        Ok(self)
    }

    /// Sets the amount to the minimum storage deposit.
    #[inline(always)]
    pub fn with_minimum_storage_deposit(mut self, byte_cost_config: ByteCostConfig) -> Self {
        self.amount = OutputBuilderAmount::MinimumStorageDeposit(byte_cost_config);
        self
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

    /// Sets the serial number to the provided value.
    #[inline(always)]
    pub fn with_serial_number(mut self, serial_number: u32) -> Self {
        self.serial_number = serial_number;
        self
    }

    /// Sets the token tag to the provided value.
    #[inline(always)]
    pub fn with_token_tag(mut self, token_tag: TokenTag) -> Self {
        self.token_tag = token_tag;
        self
    }

    /// Sets the token scheme to the provided value.
    #[inline(always)]
    pub fn with_token_scheme(mut self, token_scheme: TokenScheme) -> Self {
        self.token_scheme = token_scheme;
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
    pub fn replace_unlock_condition(mut self, unlock_condition: UnlockCondition) -> Result<Self, Error> {
        match self
            .unlock_conditions
            .iter_mut()
            .find(|u| u.kind() == unlock_condition.kind())
        {
            Some(u) => *u = unlock_condition,
            None => return Err(Error::CannotReplaceMissingField),
        }
        Ok(self)
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
    pub fn replace_feature_block(mut self, feature_block: FeatureBlock) -> Result<Self, Error> {
        match self
            .feature_blocks
            .iter_mut()
            .find(|f| f.kind() == feature_block.kind())
        {
            Some(f) => *f = feature_block,
            None => return Err(Error::CannotReplaceMissingField),
        }
        Ok(self)
    }

    ///
    #[inline(always)]
    pub fn add_immutable_feature_block(mut self, immutable_feature_block: FeatureBlock) -> Self {
        self.immutable_feature_blocks.push(immutable_feature_block);
        self
    }

    ///
    #[inline(always)]
    pub fn with_immutable_feature_blocks(
        mut self,
        immutable_feature_blocks: impl IntoIterator<Item = FeatureBlock>,
    ) -> Self {
        self.immutable_feature_blocks = immutable_feature_blocks.into_iter().collect();
        self
    }

    ///
    pub fn replace_immutable_feature_block(mut self, immutable_feature_block: FeatureBlock) -> Result<Self, Error> {
        match self
            .immutable_feature_blocks
            .iter_mut()
            .find(|f| f.kind() == immutable_feature_block.kind())
        {
            Some(f) => *f = immutable_feature_block,
            None => return Err(Error::CannotReplaceMissingField),
        }
        Ok(self)
    }

    ///
    pub fn finish(self) -> Result<FoundryOutput, Error> {
        let unlock_conditions = UnlockConditions::new(self.unlock_conditions)?;

        verify_unlock_conditions(&unlock_conditions)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        verify_allowed_feature_blocks(&feature_blocks, FoundryOutput::ALLOWED_FEATURE_BLOCKS)?;

        let immutable_feature_blocks = FeatureBlocks::new(self.immutable_feature_blocks)?;

        verify_allowed_feature_blocks(
            &immutable_feature_blocks,
            FoundryOutput::ALLOWED_IMMUTABLE_FEATURE_BLOCKS,
        )?;

        let mut output = FoundryOutput {
            amount: 1u64.try_into().map_err(Error::InvalidOutputAmount)?,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            serial_number: self.serial_number,
            token_tag: self.token_tag,
            token_scheme: self.token_scheme,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        };

        output.amount = match self.amount {
            OutputBuilderAmount::Amount(amount) => amount,
            OutputBuilderAmount::MinimumStorageDeposit(byte_cost_config) => Output::Foundry(output.clone())
                .byte_cost(&byte_cost_config)
                .try_into()
                .map_err(Error::InvalidOutputAmount)?,
        };

        Ok(output)
    }

    /// Finishes the [`FoundryOutputBuilder`] into an [`Output`].
    pub fn finish_output(self) -> Result<Output, Error> {
        Ok(Output::Foundry(self.finish()?))
    }
}

impl From<&FoundryOutput> for FoundryOutputBuilder {
    fn from(output: &FoundryOutput) -> Self {
        FoundryOutputBuilder {
            amount: OutputBuilderAmount::Amount(output.amount),
            native_tokens: output.native_tokens.to_vec(),
            serial_number: output.serial_number,
            token_tag: output.token_tag,
            token_scheme: output.token_scheme.clone(),
            unlock_conditions: output.unlock_conditions.to_vec(),
            feature_blocks: output.feature_blocks.to_vec(),
            immutable_feature_blocks: output.immutable_feature_blocks.to_vec(),
        }
    }
}

/// Describes a foundry output that is controlled by an alias.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FoundryOutput {
    // Amount of IOTA tokens held by the output.
    amount: OutputAmount,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // The serial number of the foundry with respect to the controlling alias.
    serial_number: u32,
    // Data that is always the last 12 bytes of ID of the tokens produced by this foundry.
    token_tag: TokenTag,
    token_scheme: TokenScheme,
    unlock_conditions: UnlockConditions,
    feature_blocks: FeatureBlocks,
    immutable_feature_blocks: FeatureBlocks,
}

impl FoundryOutput {
    /// The [`Output`](crate::output::Output) kind of a [`FoundryOutput`].
    pub const KIND: u8 = 5;
    /// The set of allowed [`UnlockCondition`]s for a [`FoundryOutput`].
    pub const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::IMMUTABLE_ALIAS_ADDRESS;
    /// The set of allowed [`FeatureBlock`]s for a [`FoundryOutput`].
    pub const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::METADATA;
    /// The set of allowed immutable [`FeatureBlock`]s for a [`FoundryOutput`].
    pub const ALLOWED_IMMUTABLE_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::METADATA;

    /// Creates a new [`FoundryOutput`] with a provided amount.
    #[inline(always)]
    pub fn new_with_amount(
        amount: u64,
        serial_number: u32,
        token_tag: TokenTag,
        token_scheme: TokenScheme,
    ) -> Result<Self, Error> {
        FoundryOutputBuilder::new_with_amount(amount, serial_number, token_tag, token_scheme)?.finish()
    }

    /// Creates a new [`FoundryOutput`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn new_with_minimum_storage_deposit(
        byte_cost_config: ByteCostConfig,
        serial_number: u32,
        token_tag: TokenTag,
        token_scheme: TokenScheme,
    ) -> Result<Self, Error> {
        FoundryOutputBuilder::new_with_minimum_storage_deposit(
            byte_cost_config,
            serial_number,
            token_tag,
            token_scheme,
        )?
        .finish()
    }

    /// Creates a new [`FoundryOutputBuilder`] with a provided amount.
    #[inline(always)]
    pub fn build_with_amount(
        amount: u64,
        serial_number: u32,
        token_tag: TokenTag,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        FoundryOutputBuilder::new_with_amount(amount, serial_number, token_tag, token_scheme)
    }

    /// Creates a new [`FoundryOutputBuilder`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn build_with_minimum_storage_deposit(
        byte_cost_config: ByteCostConfig,
        serial_number: u32,
        token_tag: TokenTag,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        FoundryOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config, serial_number, token_tag, token_scheme)
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
    pub fn serial_number(&self) -> u32 {
        self.serial_number
    }

    ///
    #[inline(always)]
    pub fn token_tag(&self) -> &TokenTag {
        &self.token_tag
    }

    ///
    #[inline(always)]
    pub fn token_scheme(&self) -> &TokenScheme {
        &self.token_scheme
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
    pub fn immutable_feature_blocks(&self) -> &FeatureBlocks {
        &self.immutable_feature_blocks
    }

    ///
    #[inline(always)]
    pub fn alias_address(&self) -> &AliasAddress {
        // A FoundryOutput must have an ImmutableAliasAddressUnlockCondition.
        self.unlock_conditions
            .immutable_alias_address()
            .map(|unlock_condition| unlock_condition.address())
            .unwrap()
    }

    /// Returns the [`FoundryId`] of the [`FoundryOutput`].
    pub fn id(&self) -> FoundryId {
        FoundryId::build(self.alias_address(), self.serial_number, self.token_scheme.kind())
    }

    /// Returns the [`TokenId`] of the [`FoundryOutput`].
    pub fn token_id(&self) -> TokenId {
        TokenId::build(&self.id(), &self.token_tag)
    }

    ///
    #[inline(always)]
    pub fn chain_id(&self) -> ChainId {
        ChainId::Foundry(self.id())
    }

    ///
    pub fn unlock(
        &self,
        _output_id: &OutputId,
        unlock_block: &UnlockBlock,
        inputs: &[(OutputId, &Output)],
        context: &mut ValidationContext,
    ) -> Result<(), ConflictReason> {
        let locked_address = Address::from(*self.alias_address());

        let locked_address = self.unlock_conditions().locked_address(
            &locked_address,
            context.milestone_index,
            context.milestone_timestamp,
        );

        locked_address.unlock(unlock_block, inputs, context)
    }
}

impl StateTransitionVerifier for FoundryOutput {
    fn creation(next_state: &Self, context: &ValidationContext) -> Result<(), StateTransitionError> {
        let alias_chain_id = ChainId::from(*next_state.alias_address().alias_id());

        if let (Some(Output::Alias(input_alias)), Some(Output::Alias(output_alias))) = (
            context.input_chains.get(&alias_chain_id),
            context.output_chains.get(&alias_chain_id),
        ) {
            if input_alias.foundry_counter() >= next_state.serial_number()
                || next_state.serial_number() > output_alias.foundry_counter()
            {
                return Err(StateTransitionError::InconsistentFoundrySerialNumber);
            }
        } else {
            return Err(StateTransitionError::MissingAliasForFoundry);
        }

        let token_id = next_state.token_id();
        let output_tokens = context.output_native_tokens.get(&token_id).copied().unwrap_or_default();
        let TokenScheme::Simple(ref next_token_scheme) = next_state.token_scheme;

        // No native tokens should be referenced prior to the foundry creation.
        if context.input_native_tokens.contains_key(&token_id) {
            return Err(StateTransitionError::InconsistentNativeTokensFoundryCreation);
        }

        if &output_tokens != next_token_scheme.minted_tokens() || !next_token_scheme.melted_tokens().is_zero() {
            return Err(StateTransitionError::InconsistentNativeTokensFoundryCreation);
        }

        Ok(())
    }

    fn transition(
        current_state: &Self,
        next_state: &Self,
        context: &ValidationContext,
    ) -> Result<(), StateTransitionError> {
        if current_state.alias_address() != next_state.alias_address()
            || current_state.serial_number != next_state.serial_number
            || current_state.token_tag != next_state.token_tag
            || current_state.immutable_feature_blocks != next_state.immutable_feature_blocks
        {
            return Err(StateTransitionError::MutatedImmutableField);
        }

        let token_id = next_state.token_id();
        let input_tokens = context.input_native_tokens.get(&token_id).copied().unwrap_or_default();
        let output_tokens = context.output_native_tokens.get(&token_id).copied().unwrap_or_default();
        let TokenScheme::Simple(ref current_token_scheme) = current_state.token_scheme;
        let TokenScheme::Simple(ref next_token_scheme) = next_state.token_scheme;

        if current_token_scheme.maximum_supply() != next_token_scheme.maximum_supply() {
            return Err(StateTransitionError::MutatedImmutableField);
        }

        if current_token_scheme.minted_tokens() > next_token_scheme.minted_tokens()
            || current_token_scheme.melted_tokens() > next_token_scheme.melted_tokens()
        {
            return Err(StateTransitionError::NonMonotonicallyIncreasingNativeTokens);
        }

        match input_tokens.cmp(&output_tokens) {
            Ordering::Less => {
                // Mint

                // This can't underflow as it is known that current_minted_tokens <= next_minted_tokens.
                let minted_diff = next_token_scheme.minted_tokens() - current_token_scheme.minted_tokens();
                // This can't underflow as it is known that input_tokens < output_tokens (Ordering::Less).
                let token_diff = output_tokens - input_tokens;

                if minted_diff != token_diff {
                    return Err(StateTransitionError::InconsistentNativeTokensMint);
                }

                if current_token_scheme.melted_tokens() != next_token_scheme.melted_tokens() {
                    return Err(StateTransitionError::InconsistentNativeTokensMint);
                }
            }
            Ordering::Equal => {
                // Transition

                if current_token_scheme.minted_tokens() != next_token_scheme.minted_tokens()
                    || current_token_scheme.melted_tokens() != next_token_scheme.melted_tokens()
                {
                    return Err(StateTransitionError::InconsistentNativeTokensTransition);
                }
            }
            Ordering::Greater => {
                // Melt / Burn

                if current_token_scheme.melted_tokens() != next_token_scheme.melted_tokens()
                    && current_token_scheme.minted_tokens() != next_token_scheme.minted_tokens()
                {
                    return Err(StateTransitionError::InconsistentNativeTokensMeltBurn);
                }

                // This can't underflow as it is known that current_melted_tokens <= next_melted_tokens.
                let melted_diff = next_token_scheme.melted_tokens() - current_token_scheme.melted_tokens();
                // This can't underflow as it is known that input_tokens > output_tokens (Ordering::Greater).
                let token_diff = input_tokens - output_tokens;

                if melted_diff > token_diff {
                    return Err(StateTransitionError::InconsistentNativeTokensMeltBurn);
                }
            }
        }

        Ok(())
    }

    fn destruction(current_state: &Self, context: &ValidationContext) -> Result<(), StateTransitionError> {
        let token_id = current_state.token_id();
        let input_tokens = context.input_native_tokens.get(&token_id).copied().unwrap_or_default();
        let TokenScheme::Simple(ref current_token_scheme) = current_state.token_scheme;

        // No native tokens should be referenced after the foundry destruction.
        if context.output_native_tokens.contains_key(&token_id) {
            return Err(StateTransitionError::InconsistentNativeTokensFoundryDestruction);
        }

        // This can't underflow as it is known that minted_tokens >= melted_tokens (syntactic rule).
        let minted_melted_diff = current_token_scheme.minted_tokens() - current_token_scheme.melted_tokens();

        if minted_melted_diff != input_tokens {
            return Err(StateTransitionError::InconsistentNativeTokensFoundryDestruction);
        }

        Ok(())
    }
}

impl Packable for FoundryOutput {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.serial_number.pack(packer)?;
        self.token_tag.pack(packer)?;
        self.token_scheme.pack(packer)?;
        self.unlock_conditions.pack(packer)?;
        self.feature_blocks.pack(packer)?;
        self.immutable_feature_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let amount = OutputAmount::unpack::<_, VERIFY>(unpacker).map_packable_err(Error::InvalidOutputAmount)?;
        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker)?;
        let serial_number = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let token_tag = TokenTag::unpack::<_, VERIFY>(unpacker).coerce()?;
        let token_scheme = TokenScheme::unpack::<_, VERIFY>(unpacker)?;

        let unlock_conditions = UnlockConditions::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_unlock_conditions(&unlock_conditions).map_err(UnpackError::Packable)?;
        }

        let feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_feature_blocks(&feature_blocks, FoundryOutput::ALLOWED_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        let immutable_feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_feature_blocks(
                &immutable_feature_blocks,
                FoundryOutput::ALLOWED_IMMUTABLE_FEATURE_BLOCKS,
            )
            .map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            serial_number,
            token_tag,
            token_scheme,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        })
    }
}

fn verify_unlock_conditions(unlock_conditions: &UnlockConditions) -> Result<(), Error> {
    if unlock_conditions.immutable_alias_address().is_none() {
        Err(Error::MissingAddressUnlockCondition)
    } else {
        verify_allowed_unlock_conditions(unlock_conditions, FoundryOutput::ALLOWED_UNLOCK_CONDITIONS)
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
            feature_block::dto::FeatureBlockDto, native_token::dto::NativeTokenDto, token_id::dto::TokenTagDto,
            token_scheme::dto::TokenSchemeDto, unlock_condition::dto::UnlockConditionDto,
        },
    };

    /// Describes a foundry output that is controlled by an alias.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct FoundryOutputDto {
        #[serde(rename = "type")]
        pub kind: u8,
        // Amount of IOTA tokens held by the output.
        pub amount: String,
        // Native tokens held by the output.
        #[serde(rename = "nativeTokens", skip_serializing_if = "Vec::is_empty", default)]
        pub native_tokens: Vec<NativeTokenDto>,
        // The serial number of the foundry with respect to the controlling alias.
        #[serde(rename = "serialNumber")]
        pub serial_number: u32,
        // Data that is always the last 12 bytes of ID of the tokens produced by this foundry.
        #[serde(rename = "tokenTag")]
        pub token_tag: TokenTagDto,
        #[serde(rename = "tokenScheme")]
        pub token_scheme: TokenSchemeDto,
        #[serde(rename = "unlockConditions")]
        pub unlock_conditions: Vec<UnlockConditionDto>,
        #[serde(rename = "featureBlocks", skip_serializing_if = "Vec::is_empty", default)]
        pub feature_blocks: Vec<FeatureBlockDto>,
        #[serde(rename = "immutableFeatureBlocks", skip_serializing_if = "Vec::is_empty", default)]
        pub immutable_feature_blocks: Vec<FeatureBlockDto>,
    }

    impl From<&FoundryOutput> for FoundryOutputDto {
        fn from(value: &FoundryOutput) -> Self {
            Self {
                kind: FoundryOutput::KIND,
                amount: value.amount().to_string(),
                native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
                serial_number: value.serial_number(),
                token_tag: TokenTagDto(value.token_tag().to_string()),
                token_scheme: value.token_scheme().into(),
                unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
                feature_blocks: value.feature_blocks().iter().map(Into::into).collect::<_>(),
                immutable_feature_blocks: value.immutable_feature_blocks().iter().map(Into::into).collect::<_>(),
            }
        }
    }

    impl TryFrom<&FoundryOutputDto> for FoundryOutput {
        type Error = DtoError;

        fn try_from(value: &FoundryOutputDto) -> Result<Self, Self::Error> {
            let mut builder = FoundryOutputBuilder::new_with_amount(
                value
                    .amount
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("amount"))?,
                value.serial_number,
                (&value.token_tag).try_into()?,
                (&value.token_scheme).try_into()?,
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

            for b in &value.immutable_feature_blocks {
                builder = builder.add_immutable_feature_block(b.try_into()?);
            }

            Ok(builder.finish()?)
        }
    }
}
