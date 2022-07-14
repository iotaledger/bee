// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

use packable::{
    bounded::BoundedU16,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
    Packable,
};

use crate::{
    address::{Address, AliasAddress},
    output::{
        feature::{verify_allowed_features, Feature, FeatureFlags, Features},
        unlock_condition::{verify_allowed_unlock_conditions, UnlockCondition, UnlockConditionFlags, UnlockConditions},
        AliasId, ByteCost, ByteCostConfig, ChainId, NativeToken, NativeTokens, Output, OutputAmount,
        OutputBuilderAmount, OutputId, StateTransitionError, StateTransitionVerifier,
    },
    semantic::{ConflictReason, ValidationContext},
    unlock::Unlock,
    Error,
};

///
#[derive(Clone)]
#[must_use]
pub struct AliasOutputBuilder {
    amount: OutputBuilderAmount,
    native_tokens: Vec<NativeToken>,
    alias_id: AliasId,
    state_index: Option<u32>,
    state_metadata: Vec<u8>,
    foundry_counter: Option<u32>,
    unlock_conditions: Vec<UnlockCondition>,
    features: Vec<Feature>,
    immutable_features: Vec<Feature>,
}

impl AliasOutputBuilder {
    /// Creates an [`AliasOutputBuilder`] with a provided amount.
    pub fn new_with_amount(amount: u64, alias_id: AliasId) -> Result<AliasOutputBuilder, Error> {
        Self::new(
            OutputBuilderAmount::Amount(amount.try_into().map_err(Error::InvalidOutputAmount)?),
            alias_id,
        )
    }

    /// Creates an [`AliasOutputBuilder`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    pub fn new_with_minimum_storage_deposit(
        byte_cost_config: ByteCostConfig,
        alias_id: AliasId,
    ) -> Result<AliasOutputBuilder, Error> {
        Self::new(OutputBuilderAmount::MinimumStorageDeposit(byte_cost_config), alias_id)
    }

    fn new(amount: OutputBuilderAmount, alias_id: AliasId) -> Result<AliasOutputBuilder, Error> {
        Ok(Self {
            amount,
            native_tokens: Vec::new(),
            alias_id,
            state_index: None,
            state_metadata: Vec::new(),
            foundry_counter: None,
            unlock_conditions: Vec::new(),
            features: Vec::new(),
            immutable_features: Vec::new(),
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

    /// Sets the alias ID to the provided value.
    #[inline(always)]
    pub fn with_alias_id(mut self, alias_id: AliasId) -> Self {
        self.alias_id = alias_id;
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
    pub fn add_feature(mut self, feature: Feature) -> Self {
        self.features.push(feature);
        self
    }

    ///
    #[inline(always)]
    pub fn with_features(mut self, features: impl IntoIterator<Item = Feature>) -> Self {
        self.features = features.into_iter().collect();
        self
    }

    ///
    pub fn replace_feature(mut self, feature: Feature) -> Result<Self, Error> {
        match self.features.iter_mut().find(|f| f.kind() == feature.kind()) {
            Some(f) => *f = feature,
            None => return Err(Error::CannotReplaceMissingField),
        }
        Ok(self)
    }

    ///
    #[inline(always)]
    pub fn add_immutable_feature(mut self, immutable_feature: Feature) -> Self {
        self.immutable_features.push(immutable_feature);
        self
    }

    ///
    #[inline(always)]
    pub fn with_immutable_features(mut self, immutable_features: impl IntoIterator<Item = Feature>) -> Self {
        self.immutable_features = immutable_features.into_iter().collect();
        self
    }

    ///
    pub fn replace_immutable_feature(mut self, immutable_feature: Feature) -> Result<Self, Error> {
        match self
            .immutable_features
            .iter_mut()
            .find(|f| f.kind() == immutable_feature.kind())
        {
            Some(f) => *f = immutable_feature,
            None => return Err(Error::CannotReplaceMissingField),
        }
        Ok(self)
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

        let features = Features::new(self.features)?;

        verify_allowed_features(&features, AliasOutput::ALLOWED_FEATURES)?;

        let immutable_features = Features::new(self.immutable_features)?;

        verify_allowed_features(&immutable_features, AliasOutput::ALLOWED_IMMUTABLE_FEATURES)?;

        let mut output = AliasOutput {
            amount: 1u64.try_into().map_err(Error::InvalidOutputAmount)?,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            alias_id: self.alias_id,
            state_index,
            state_metadata,
            foundry_counter,
            unlock_conditions,
            features,
            immutable_features,
        };

        output.amount = match self.amount {
            OutputBuilderAmount::Amount(amount) => amount,
            OutputBuilderAmount::MinimumStorageDeposit(byte_cost_config) => Output::Alias(output.clone())
                .byte_cost(&byte_cost_config)
                .try_into()
                .map_err(Error::InvalidOutputAmount)?,
        };

        Ok(output)
    }

    /// Finishes the [`AliasOutputBuilder`] into an [`Output`].
    pub fn finish_output(self) -> Result<Output, Error> {
        Ok(Output::Alias(self.finish()?))
    }
}

impl From<&AliasOutput> for AliasOutputBuilder {
    fn from(output: &AliasOutput) -> Self {
        AliasOutputBuilder {
            amount: OutputBuilderAmount::Amount(output.amount),
            native_tokens: output.native_tokens.to_vec(),
            alias_id: output.alias_id,
            state_index: Some(output.state_index),
            state_metadata: output.state_metadata.to_vec(),
            foundry_counter: Some(output.foundry_counter),
            unlock_conditions: output.unlock_conditions.to_vec(),
            features: output.features.to_vec(),
            immutable_features: output.immutable_features.to_vec(),
        }
    }
}

pub(crate) type StateMetadataLength = BoundedU16<0, { AliasOutput::STATE_METADATA_LENGTH_MAX }>;

/// Describes an alias account in the ledger that can be controlled by the state and governance controllers.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    features: Features,
    //
    immutable_features: Features,
}

impl AliasOutput {
    /// The [`Output`](crate::output::Output) kind of an [`AliasOutput`].
    pub const KIND: u8 = 4;
    /// Maximum possible length in bytes of the state metadata.
    pub const STATE_METADATA_LENGTH_MAX: u16 = 8192;
    /// The set of allowed [`UnlockCondition`]s for an [`AliasOutput`].
    pub const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags =
        UnlockConditionFlags::STATE_CONTROLLER_ADDRESS.union(UnlockConditionFlags::GOVERNOR_ADDRESS);
    /// The set of allowed [`Feature`]s for an [`AliasOutput`].
    pub const ALLOWED_FEATURES: FeatureFlags = FeatureFlags::SENDER.union(FeatureFlags::METADATA);
    /// The set of allowed immutable [`Feature`]s for an [`AliasOutput`].
    pub const ALLOWED_IMMUTABLE_FEATURES: FeatureFlags = FeatureFlags::ISSUER.union(FeatureFlags::METADATA);

    /// Creates a new [`AliasOutput`] with a provided amount.
    #[inline(always)]
    pub fn new_with_amount(amount: u64, alias_id: AliasId) -> Result<Self, Error> {
        AliasOutputBuilder::new_with_amount(amount, alias_id)?.finish()
    }

    /// Creates a new [`AliasOutput`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn new_with_minimum_storage_deposit(
        byte_cost_config: ByteCostConfig,
        alias_id: AliasId,
    ) -> Result<Self, Error> {
        AliasOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config, alias_id)?.finish()
    }

    /// Creates a new [`AliasOutputBuilder`] with a provided amount.
    #[inline(always)]
    pub fn build_with_amount(amount: u64, alias_id: AliasId) -> Result<AliasOutputBuilder, Error> {
        AliasOutputBuilder::new_with_amount(amount, alias_id)
    }

    /// Creates a new [`AliasOutputBuilder`] with a provided byte cost config.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn build_with_minimum_storage_deposit(
        byte_cost_config: ByteCostConfig,
        alias_id: AliasId,
    ) -> Result<AliasOutputBuilder, Error> {
        AliasOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config, alias_id)
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
    pub fn features(&self) -> &Features {
        &self.features
    }

    ///
    #[inline(always)]
    pub fn immutable_features(&self) -> &Features {
        &self.immutable_features
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

    ///
    pub fn unlock(
        &self,
        output_id: &OutputId,
        unlock: &Unlock,
        inputs: &[(OutputId, &Output)],
        context: &mut ValidationContext,
    ) -> Result<(), ConflictReason> {
        let alias_id = if self.alias_id().is_null() {
            AliasId::from(*output_id)
        } else {
            *self.alias_id()
        };
        let next_state = context.output_chains.get(&ChainId::from(alias_id));

        let locked_address = self.unlock_conditions().locked_address(
            match next_state {
                Some(Output::Alias(next_state)) => {
                    if self.state_index() == next_state.state_index() {
                        self.governor_address()
                    } else {
                        self.state_controller_address()
                    }
                }
                None => self.governor_address(),
                // The next state can only be an alias output since it is identified by an alias chain identifier.
                Some(_) => unreachable!(),
            },
            context.milestone_timestamp,
        );

        locked_address.unlock(unlock, inputs, context)?;

        context
            .unlocked_addresses
            .insert(Address::from(AliasAddress::from(alias_id)));

        Ok(())
    }
}

impl StateTransitionVerifier for AliasOutput {
    fn creation(next_state: &Self, context: &ValidationContext) -> Result<(), StateTransitionError> {
        if !next_state.alias_id.is_null() {
            return Err(StateTransitionError::NonZeroCreatedId);
        }

        if let Some(issuer) = next_state.immutable_features().issuer() {
            if !context.unlocked_addresses.contains(issuer.address()) {
                return Err(StateTransitionError::IssuerNotUnlocked);
            }
        }

        Ok(())
    }

    fn transition(
        current_state: &Self,
        next_state: &Self,
        context: &ValidationContext,
    ) -> Result<(), StateTransitionError> {
        if current_state.immutable_features != next_state.immutable_features {
            return Err(StateTransitionError::MutatedImmutableField);
        }

        if next_state.state_index == current_state.state_index + 1 {
            // State transition.
            if current_state.state_controller_address() != next_state.state_controller_address()
                || current_state.governor_address() != next_state.governor_address()
                || current_state.features.metadata() != next_state.features.metadata()
            {
                return Err(StateTransitionError::MutatedFieldWithoutRights);
            }

            let created_foundries = context.essence.outputs().iter().filter_map(|output| {
                if let Output::Foundry(foundry) = output {
                    if foundry.alias_address().alias_id() == &next_state.alias_id
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
                created_foundries_count += 1;

                if foundry.serial_number() != current_state.foundry_counter + created_foundries_count {
                    return Err(StateTransitionError::UnsortedCreatedFoundries);
                }
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
        self.features.pack(packer)?;
        self.immutable_features.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let amount = OutputAmount::unpack::<_, VERIFY>(unpacker).map_packable_err(Error::InvalidOutputAmount)?;
        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker)?;
        let alias_id = AliasId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let state_index = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let state_metadata = BoxedSlicePrefix::<u8, StateMetadataLength>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::InvalidStateMetadataLength(err.into_prefix_err().into()))?;

        let foundry_counter = u32::unpack::<_, VERIFY>(unpacker).coerce()?;

        if VERIFY {
            verify_index_counter(&alias_id, state_index, foundry_counter).map_err(UnpackError::Packable)?;
        }

        let unlock_conditions = UnlockConditions::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_unlock_conditions(&unlock_conditions, &alias_id).map_err(UnpackError::Packable)?;
        }

        let features = Features::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_features(&features, AliasOutput::ALLOWED_FEATURES).map_err(UnpackError::Packable)?;
        }

        let immutable_features = Features::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_features(&immutable_features, AliasOutput::ALLOWED_IMMUTABLE_FEATURES)
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
            features,
            immutable_features,
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

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        error::dto::DtoError,
        output::{
            alias_id::dto::AliasIdDto, dto::OutputBuilderAmountDto, feature::dto::FeatureDto,
            native_token::dto::NativeTokenDto, unlock_condition::dto::UnlockConditionDto,
        },
    };

    /// Describes an alias account in the ledger that can be controlled by the state and governance controllers.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct AliasOutputDto {
        #[serde(rename = "type")]
        pub kind: u8,
        // Amount of IOTA tokens held by the output.
        pub amount: String,
        // Native tokens held by the output.
        #[serde(rename = "nativeTokens", skip_serializing_if = "Vec::is_empty", default)]
        pub native_tokens: Vec<NativeTokenDto>,
        // Unique identifier of the alias.
        #[serde(rename = "aliasId")]
        pub alias_id: AliasIdDto,
        // A counter that must increase by 1 every time the alias is state transitioned.
        #[serde(rename = "stateIndex")]
        pub state_index: u32,
        // Metadata that can only be changed by the state controller.
        #[serde(rename = "stateMetadata", skip_serializing_if = "String::is_empty", default)]
        pub state_metadata: String,
        // A counter that denotes the number of foundries created by this alias account.
        #[serde(rename = "foundryCounter")]
        pub foundry_counter: u32,
        //
        #[serde(rename = "unlockConditions")]
        pub unlock_conditions: Vec<UnlockConditionDto>,
        //
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub features: Vec<FeatureDto>,
        //
        #[serde(rename = "immutableFeatures", skip_serializing_if = "Vec::is_empty", default)]
        pub immutable_features: Vec<FeatureDto>,
    }

    impl From<&AliasOutput> for AliasOutputDto {
        fn from(value: &AliasOutput) -> Self {
            Self {
                kind: AliasOutput::KIND,
                amount: value.amount().to_string(),
                native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
                alias_id: AliasIdDto(value.alias_id().to_string()),
                state_index: value.state_index(),
                state_metadata: prefix_hex::encode(value.state_metadata()),
                foundry_counter: value.foundry_counter(),
                unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
                features: value.features().iter().map(Into::into).collect::<_>(),
                immutable_features: value.immutable_features().iter().map(Into::into).collect::<_>(),
            }
        }
    }

    impl TryFrom<&AliasOutputDto> for AliasOutput {
        type Error = DtoError;

        fn try_from(value: &AliasOutputDto) -> Result<Self, Self::Error> {
            let mut builder = AliasOutputBuilder::new_with_amount(
                value
                    .amount
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("amount"))?,
                (&value.alias_id).try_into()?,
            )?;

            builder = builder.with_state_index(value.state_index);

            if !value.state_metadata.is_empty() {
                builder = builder.with_state_metadata(
                    prefix_hex::decode(&value.state_metadata).map_err(|_| DtoError::InvalidField("state_metadata"))?,
                );
            }

            builder = builder.with_foundry_counter(value.foundry_counter);

            for t in &value.native_tokens {
                builder = builder.add_native_token(t.try_into()?);
            }

            for b in &value.unlock_conditions {
                builder = builder.add_unlock_condition(b.try_into()?);
            }

            for b in &value.features {
                builder = builder.add_feature(b.try_into()?);
            }

            for b in &value.immutable_features {
                builder = builder.add_immutable_feature(b.try_into()?);
            }

            Ok(builder.finish()?)
        }
    }

    impl AliasOutput {
        #[allow(clippy::too_many_arguments)]
        pub fn from_dtos(
            amount: OutputBuilderAmountDto,
            native_tokens: Option<Vec<NativeTokenDto>>,
            alias_id: &AliasIdDto,
            state_index: Option<u32>,
            state_metadata: Option<Vec<u8>>,
            foundry_counter: Option<u32>,
            unlock_conditions: Vec<UnlockConditionDto>,
            features: Option<Vec<FeatureDto>>,
            immutable_features: Option<Vec<FeatureDto>>,
        ) -> Result<AliasOutput, DtoError> {
            let alias_id = AliasId::try_from(alias_id)?;

            let mut builder = match amount {
                OutputBuilderAmountDto::Amount(amount) => AliasOutputBuilder::new_with_amount(
                    amount.parse().map_err(|_| DtoError::InvalidField("amount"))?,
                    alias_id,
                )?,
                OutputBuilderAmountDto::MinimumStorageDeposit(byte_cost_config) => {
                    AliasOutputBuilder::new_with_minimum_storage_deposit(byte_cost_config, alias_id)?
                }
            };

            if let Some(native_tokens) = native_tokens {
                let native_tokens = native_tokens
                    .iter()
                    .map(NativeToken::try_from)
                    .collect::<Result<Vec<NativeToken>, DtoError>>()?;
                builder = builder.with_native_tokens(native_tokens);
            }

            if let Some(state_index) = state_index {
                builder = builder.with_state_index(state_index);
            }

            if let Some(state_metadata) = state_metadata {
                builder = builder.with_state_metadata(state_metadata);
            }

            if let Some(foundry_counter) = foundry_counter {
                builder = builder.with_foundry_counter(foundry_counter);
            }

            let unlock_conditions = unlock_conditions
                .iter()
                .map(UnlockCondition::try_from)
                .collect::<Result<Vec<UnlockCondition>, DtoError>>()?;
            builder = builder.with_unlock_conditions(unlock_conditions);

            if let Some(features) = features {
                let features = features
                    .iter()
                    .map(Feature::try_from)
                    .collect::<Result<Vec<Feature>, DtoError>>()?;
                builder = builder.with_features(features);
            }

            if let Some(immutable_features) = immutable_features {
                let immutable_features = immutable_features
                    .iter()
                    .map(Feature::try_from)
                    .collect::<Result<Vec<Feature>, DtoError>>()?;
                builder = builder.with_immutable_features(immutable_features);
            }

            Ok(builder.finish()?)
        }
    }
}
