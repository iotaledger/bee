// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{verify_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{verify_allowed_unlock_conditions, UnlockCondition, UnlockConditionFlags, UnlockConditions},
        AliasId, ChainId, NativeToken, NativeTokens, Output, OutputAmount, StateTransition, StateTransitionError,
    },
    semantic::ValidationContext,
    Error,
};

use packable::{
    bounded::BoundedU16,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
    Packable,
};

use alloc::vec::Vec;

///
#[must_use]
pub struct AliasOutputBuilder {
    amount: OutputAmount,
    native_tokens: Vec<NativeToken>,
    alias_id: AliasId,
    state_index: Option<u32>,
    state_metadata: Vec<u8>,
    foundry_counter: Option<u32>,
    unlock_conditions: Vec<UnlockCondition>,
    feature_blocks: Vec<FeatureBlock>,
    immutable_feature_blocks: Vec<FeatureBlock>,
}

impl AliasOutputBuilder {
    ///
    pub fn new(amount: u64, alias_id: AliasId) -> Result<AliasOutputBuilder, Error> {
        Ok(Self {
            amount: amount.try_into().map_err(Error::InvalidOutputAmount)?,
            native_tokens: Vec::new(),
            alias_id,
            state_index: None,
            state_metadata: Vec::new(),
            foundry_counter: None,
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
    pub fn with_state_index(mut self, state_index: u32) -> Self {
        self.state_index.replace(state_index);
        self
    }

    ///
    #[inline(always)]
    pub fn with_state_metadata(mut self, state_metadata: Vec<u8>) -> Self {
        self.state_metadata = state_metadata;
        self
    }

    ///
    #[inline(always)]
    pub fn with_foundry_counter(mut self, foundry_counter: u32) -> Self {
        self.foundry_counter.replace(foundry_counter);
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
    pub fn finish(self) -> Result<AliasOutput, Error> {
        let state_index = self.state_index.unwrap_or(0);
        let foundry_counter = self.foundry_counter.unwrap_or(0);

        let state_metadata = self
            .state_metadata
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidStateMetadataLength)?;

        verify_index_counter(&self.alias_id, state_index, foundry_counter)?;

        let unlock_conditions = UnlockConditions::new(self.unlock_conditions)?;

        verify_unlock_conditions(&unlock_conditions, &self.alias_id)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        verify_allowed_feature_blocks(&feature_blocks, AliasOutput::ALLOWED_FEATURE_BLOCKS)?;

        let immutable_feature_blocks = FeatureBlocks::new(self.immutable_feature_blocks)?;

        verify_allowed_feature_blocks(&immutable_feature_blocks, AliasOutput::ALLOWED_IMMUTABLE_FEATURE_BLOCKS)?;

        Ok(AliasOutput {
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            alias_id: self.alias_id,
            state_index,
            state_metadata,
            foundry_counter,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        })
    }
}

pub(crate) type StateMetadataLength = BoundedU16<0, { AliasOutput::STATE_METADATA_LENGTH_MAX }>;

/// Describes an alias account in the ledger that can be controlled by the state and governance controllers.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct AliasOutput {
    // Amount of IOTA tokens held by the output.
    amount: OutputAmount,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // Unique identifier of the alias.
    alias_id: AliasId,
    // A counter that must increase by 1 every time the alias is state transitioned.
    state_index: u32,
    // Metadata that can only be changed by the state controller.
    state_metadata: BoxedSlicePrefix<u8, StateMetadataLength>,
    // A counter that denotes the number of foundries created by this alias account.
    foundry_counter: u32,
    unlock_conditions: UnlockConditions,
    //
    feature_blocks: FeatureBlocks,
    //
    immutable_feature_blocks: FeatureBlocks,
}

impl AliasOutput {
    /// The [`Output`](crate::output::Output) kind of an [`AliasOutput`].
    pub const KIND: u8 = 4;
    /// Maximum possible length in bytes of the state metadata.
    pub const STATE_METADATA_LENGTH_MAX: u16 = 8192;
    /// The set of allowed [`UnlockCondition`]s for an [`AliasOutput`].
    pub const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags =
        UnlockConditionFlags::STATE_CONTROLLER_ADDRESS.union(UnlockConditionFlags::GOVERNOR_ADDRESS);
    /// The set of allowed [`FeatureBlock`]s for an [`AliasOutput`].
    pub const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER.union(FeatureBlockFlags::METADATA);
    /// The set of allowed immutable [`FeatureBlock`]s for an [`AliasOutput`].
    pub const ALLOWED_IMMUTABLE_FEATURE_BLOCKS: FeatureBlockFlags =
        FeatureBlockFlags::ISSUER.union(FeatureBlockFlags::METADATA);

    /// Creates a new [`AliasOutput`].
    #[inline(always)]
    pub fn new(amount: u64, alias_id: AliasId) -> Result<Self, Error> {
        AliasOutputBuilder::new(amount, alias_id)?.finish()
    }

    /// Creates a new [`AliasOutputBuilder`].
    #[inline(always)]
    pub fn build(amount: u64, alias_id: AliasId) -> Result<AliasOutputBuilder, Error> {
        AliasOutputBuilder::new(amount, alias_id)
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
    pub fn alias_id(&self) -> &AliasId {
        &self.alias_id
    }

    ///
    #[inline(always)]
    pub fn state_index(&self) -> u32 {
        self.state_index
    }

    ///
    #[inline(always)]
    pub fn state_metadata(&self) -> &[u8] {
        &self.state_metadata
    }

    ///
    #[inline(always)]
    pub fn foundry_counter(&self) -> u32 {
        self.foundry_counter
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
    pub fn state_controller_address(&self) -> &Address {
        // An AliasOutput must have a StateControllerAddressUnlockCondition.
        self.unlock_conditions
            .state_controller_address()
            .map(|unlock_condition| unlock_condition.address())
            .unwrap()
    }

    ///
    #[inline(always)]
    pub fn governor_address(&self) -> &Address {
        // An AliasOutput must have a GovernorAddressUnlockCondition.
        self.unlock_conditions
            .governor_address()
            .map(|unlock_condition| unlock_condition.address())
            .unwrap()
    }

    ///
    #[inline(always)]
    pub fn chain_id(&self) -> ChainId {
        ChainId::Alias(self.alias_id)
    }
}

impl StateTransition for AliasOutput {
    fn creation(next_state: &Self, _context: &ValidationContext) -> Result<(), StateTransitionError> {
        if !next_state.alias_id.is_null() {
            return Err(StateTransitionError::NonZeroCreatedId);
        }

        Ok(())
    }

    fn transition(
        current_state: &Self,
        next_state: &Self,
        context: &ValidationContext,
    ) -> Result<(), StateTransitionError> {
        if current_state.immutable_feature_blocks != next_state.immutable_feature_blocks {
            return Err(StateTransitionError::MutatedImmutableField);
        }

        if next_state.state_index == current_state.state_index + 1 {
            // State transition.
            if current_state.state_controller_address() != next_state.state_controller_address()
                || current_state.governor_address() != next_state.governor_address()
                || current_state.feature_blocks.metadata() != next_state.feature_blocks.metadata()
            {
                return Err(StateTransitionError::MutatedFieldWithoutRights);
            }

            let created_foundries = context.essence.outputs().iter().filter_map(|output| {
                if let Output::Foundry(foundry) = output {
                    if foundry.alias_address().alias_id() == &current_state.alias_id
                        && !context.input_chains.contains_key(&foundry.chain_id())
                    {
                        Some(foundry)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            let mut created_foundries_count = 0;

            for foundry in created_foundries {
                if foundry.serial_number() != current_state.foundry_counter + created_foundries_count {
                    return Err(StateTransitionError::UnsortedCreatedFoundries);
                }

                created_foundries_count += 1;
            }

            if current_state.foundry_counter + created_foundries_count != next_state.foundry_counter {
                return Err(StateTransitionError::InconsistentCreatedFoundriesCount);
            }
        } else if next_state.state_index == current_state.state_index {
            // Governance transition.
            if current_state.amount != next_state.amount
                || current_state.native_tokens != next_state.native_tokens
                || current_state.state_index != next_state.state_index
                || current_state.state_metadata != next_state.state_metadata
                || current_state.foundry_counter != next_state.foundry_counter
            {
                return Err(StateTransitionError::MutatedFieldWithoutRights);
            }
        } else {
            return Err(StateTransitionError::UnsupportedStateIndexOperation {
                current_state: current_state.state_index,
                next_state: next_state.state_index,
            });
        }

        Ok(())
    }

    fn destruction(_current_state: &Self, _context: &ValidationContext) -> Result<(), StateTransitionError> {
        Ok(())
    }
}

impl Packable for AliasOutput {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.alias_id.pack(packer)?;
        self.state_index.pack(packer)?;
        self.state_metadata.pack(packer)?;
        self.foundry_counter.pack(packer)?;
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
        let alias_id = AliasId::unpack::<_, VERIFY>(unpacker).infallible()?;
        let state_index = u32::unpack::<_, VERIFY>(unpacker).infallible()?;
        let state_metadata = BoxedSlicePrefix::<u8, StateMetadataLength>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::InvalidStateMetadataLength(err.into_prefix_err().into()))?;

        let foundry_counter = u32::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            verify_index_counter(&alias_id, state_index, foundry_counter).map_err(UnpackError::Packable)?;
        }

        let unlock_conditions = UnlockConditions::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_unlock_conditions(&unlock_conditions, &alias_id).map_err(UnpackError::Packable)?;
        }

        let feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_feature_blocks(&feature_blocks, AliasOutput::ALLOWED_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        let immutable_feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_feature_blocks(&immutable_feature_blocks, AliasOutput::ALLOWED_IMMUTABLE_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            alias_id,
            state_index,
            state_metadata,
            foundry_counter,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        })
    }
}

#[inline]
fn verify_index_counter(alias_id: &AliasId, state_index: u32, foundry_counter: u32) -> Result<(), Error> {
    if alias_id.is_null() && (state_index != 0 || foundry_counter != 0) {
        Err(Error::NonZeroStateIndexOrFoundryCounter)
    } else {
        Ok(())
    }
}

fn verify_unlock_conditions(unlock_conditions: &UnlockConditions, alias_id: &AliasId) -> Result<(), Error> {
    if let Some(unlock_condition) = unlock_conditions.state_controller_address() {
        if let Address::Alias(alias_address) = unlock_condition.address() {
            if alias_address.alias_id() == alias_id {
                return Err(Error::SelfControlledAliasOutput(*alias_id));
            }
        }
    } else {
        return Err(Error::MissingStateControllerUnlockCondition);
    }

    if let Some(unlock_condition) = unlock_conditions.governor_address() {
        if let Address::Alias(alias_address) = unlock_condition.address() {
            if alias_address.alias_id() == alias_id {
                return Err(Error::SelfControlledAliasOutput(*alias_id));
            }
        }
    } else {
        return Err(Error::MissingGovernorUnlockCondition);
    }

    verify_allowed_unlock_conditions(unlock_conditions, AliasOutput::ALLOWED_UNLOCK_CONDITIONS)
}
