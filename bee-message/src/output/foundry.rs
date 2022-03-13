// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::AliasAddress,
    output::{
        feature_block::{verify_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{verify_allowed_unlock_conditions, UnlockCondition, UnlockConditionFlags, UnlockConditions},
        ChainId, FoundryId, NativeToken, NativeTokens, OutputAmount, StateTransition, StateTransitionError, TokenId,
        TokenScheme, TokenTag,
    },
    semantic::ValidationContext,
    Error,
};

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};
use primitive_types::U256;

use alloc::vec::Vec;

///
#[must_use]
pub struct FoundryOutputBuilder {
    amount: OutputAmount,
    native_tokens: Vec<NativeToken>,
    serial_number: u32,
    token_tag: TokenTag,
    minted_tokens: U256,
    melted_tokens: U256,
    maximum_supply: U256,
    token_scheme: TokenScheme,
    unlock_conditions: Vec<UnlockCondition>,
    feature_blocks: Vec<FeatureBlock>,
    immutable_feature_blocks: Vec<FeatureBlock>,
}

impl FoundryOutputBuilder {
    ///
    pub fn new(
        amount: u64,
        serial_number: u32,
        token_tag: TokenTag,
        minted_tokens: U256,
        melted_tokens: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        verify_supply(&minted_tokens, &melted_tokens, &maximum_supply)?;

        Ok(Self {
            amount: amount.try_into().map_err(Error::InvalidOutputAmount)?,
            native_tokens: Vec::new(),
            serial_number,
            token_tag,
            minted_tokens,
            melted_tokens,
            maximum_supply,
            token_scheme,
            unlock_conditions: Vec::new(),
            feature_blocks: Vec::new(),
            immutable_feature_blocks: Vec::new(),
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

        Ok(FoundryOutput {
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            serial_number: self.serial_number,
            token_tag: self.token_tag,
            minted_tokens: self.minted_tokens,
            melted_tokens: self.melted_tokens,
            maximum_supply: self.maximum_supply,
            token_scheme: self.token_scheme,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        })
    }
}

/// Describes a foundry output that is controlled by an alias.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FoundryOutput {
    // Amount of IOTA tokens held by the output.
    amount: OutputAmount,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // The serial number of the foundry with respect to the controlling alias.
    serial_number: u32,
    // Data that is always the last 12 bytes of ID of the tokens produced by this foundry.
    token_tag: TokenTag,
    // Amount of tokens minted by this foundry.
    minted_tokens: U256,
    // Amount of tokens melted by this foundry.
    melted_tokens: U256,
    // Maximum supply of tokens controlled by this foundry.
    maximum_supply: U256,
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

    /// Creates a new [`FoundryOutput`].
    #[inline(always)]
    pub fn new(
        amount: u64,
        serial_number: u32,
        token_tag: TokenTag,
        minted_tokens: U256,
        melted_tokens: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<Self, Error> {
        FoundryOutputBuilder::new(
            amount,
            serial_number,
            token_tag,
            minted_tokens,
            melted_tokens,
            maximum_supply,
            token_scheme,
        )?
        .finish()
    }

    /// Creates a new [`FoundryOutputBuilder`].
    #[inline(always)]
    pub fn build(
        amount: u64,
        serial_number: u32,
        token_tag: TokenTag,
        minted_tokens: U256,
        melted_tokens: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        FoundryOutputBuilder::new(
            amount,
            serial_number,
            token_tag,
            minted_tokens,
            melted_tokens,
            maximum_supply,
            token_scheme,
        )
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
    pub fn minted_tokens(&self) -> &U256 {
        &self.minted_tokens
    }

    ///
    #[inline(always)]
    pub fn melted_tokens(&self) -> &U256 {
        &self.melted_tokens
    }

    ///
    #[inline(always)]
    pub fn maximum_supply(&self) -> &U256 {
        &self.maximum_supply
    }

    ///
    #[inline(always)]
    pub fn token_scheme(&self) -> TokenScheme {
        self.token_scheme
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
        FoundryId::build(self.alias_address(), self.serial_number, self.token_scheme)
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
}

impl StateTransition for FoundryOutput {
    fn creation(next_state: &Self, context: &ValidationContext) -> Result<(), StateTransitionError> {
        let alias_chain_id = ChainId::from(*next_state.alias_address().alias_id());

        if let (Some(_input_alias), Some(_output_alias)) = (
            context.input_chains.get(&alias_chain_id),
            context.output_chains.get(&alias_chain_id),
        ) {
            // TODO check serial
        }

        Ok(())
    }

    fn transition(
        current_state: &Self,
        next_state: &Self,
        _context: &ValidationContext,
    ) -> Result<(), StateTransitionError> {
        if current_state.maximum_supply != next_state.maximum_supply
            || current_state.alias_address() != next_state.alias_address()
            || current_state.serial_number != next_state.serial_number
            || current_state.token_tag != next_state.token_tag
            || current_state.token_scheme != next_state.token_scheme
            || current_state.immutable_feature_blocks != next_state.immutable_feature_blocks
        {
            return Err(StateTransitionError::MutatedImmutableField);
        }

        Ok(())
    }

    fn destruction(_current_state: &Self, _context: &ValidationContext) -> Result<(), StateTransitionError> {
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
        self.minted_tokens.pack(packer)?;
        self.melted_tokens.pack(packer)?;
        self.maximum_supply.pack(packer)?;
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
        let serial_number = u32::unpack::<_, VERIFY>(unpacker).infallible()?;
        let token_tag = TokenTag::unpack::<_, VERIFY>(unpacker).infallible()?;
        let minted_tokens = U256::unpack::<_, VERIFY>(unpacker).infallible()?;
        let melted_tokens = U256::unpack::<_, VERIFY>(unpacker).infallible()?;
        let maximum_supply = U256::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            verify_supply(&minted_tokens, &melted_tokens, &maximum_supply).map_err(UnpackError::Packable)?;
        }

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
            minted_tokens,
            melted_tokens,
            maximum_supply,
            token_scheme,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        })
    }
}

#[inline]
fn verify_supply(minted_tokens: &U256, melted_tokens: &U256, maximum_supply: &U256) -> Result<(), Error> {
    if maximum_supply.is_zero() || minted_tokens > maximum_supply || melted_tokens > minted_tokens {
        return Err(Error::InvalidFoundryOutputSupply {
            minted: *minted_tokens,
            melted: *melted_tokens,
            max: *maximum_supply,
        });
    }

    Ok(())
}

fn verify_unlock_conditions(unlock_conditions: &UnlockConditions) -> Result<(), Error> {
    if unlock_conditions.immutable_alias_address().is_none() {
        Err(Error::MissingAddressUnlockCondition)
    } else {
        verify_allowed_unlock_conditions(unlock_conditions, FoundryOutput::ALLOWED_UNLOCK_CONDITIONS)
    }
}
