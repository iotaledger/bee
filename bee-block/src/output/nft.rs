// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use crate::{
    address::{Address, NftAddress},
    output::{
        feature::{verify_allowed_features, Feature, FeatureFlags, Features},
        unlock_condition::{verify_allowed_unlock_conditions, UnlockCondition, UnlockConditionFlags, UnlockConditions},
        verify_output_amount, ChainId, NativeToken, NativeTokens, NftId, Output, OutputBuilderAmount, OutputId, Rent,
        RentStructure, StateTransitionError, StateTransitionVerifier,
    },
    protocol::ProtocolParameters,
    semantic::{ConflictReason, ValidationContext},
    unlock::Unlock,
    Error,
};

///
#[derive(Clone)]
#[must_use]
pub struct NftOutputBuilder {
    amount: OutputBuilderAmount,
    native_tokens: Vec<NativeToken>,
    nft_id: NftId,
    unlock_conditions: Vec<UnlockCondition>,
    features: Vec<Feature>,
    immutable_features: Vec<Feature>,
}

impl NftOutputBuilder {
    /// Creates an [`NftOutputBuilder`] with a provided amount.
    pub fn new_with_amount(amount: u64, nft_id: NftId) -> Result<NftOutputBuilder, Error> {
        Self::new(OutputBuilderAmount::Amount(amount), nft_id)
    }

    /// Creates an [`NftOutputBuilder`] with a provided rent structure.
    /// The amount will be set to the minimum storage deposit.
    pub fn new_with_minimum_storage_deposit(
        rent_structure: RentStructure,
        nft_id: NftId,
    ) -> Result<NftOutputBuilder, Error> {
        Self::new(OutputBuilderAmount::MinimumStorageDeposit(rent_structure), nft_id)
    }

    fn new(amount: OutputBuilderAmount, nft_id: NftId) -> Result<NftOutputBuilder, Error> {
        Ok(Self {
            amount,
            native_tokens: Vec::new(),
            nft_id,
            unlock_conditions: Vec::new(),
            features: Vec::new(),
            immutable_features: Vec::new(),
        })
    }

    /// Sets the amount to the provided value.
    #[inline(always)]
    pub fn with_amount(mut self, amount: u64) -> Result<Self, Error> {
        self.amount = OutputBuilderAmount::Amount(amount);
        Ok(self)
    }

    /// Sets the amount to the minimum storage deposit.
    #[inline(always)]
    pub fn with_minimum_storage_deposit(mut self, rent_structure: RentStructure) -> Self {
        self.amount = OutputBuilderAmount::MinimumStorageDeposit(rent_structure);
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

    /// Sets the NFT ID to the provided value.
    #[inline(always)]
    pub fn with_nft_id(mut self, nft_id: NftId) -> Self {
        self.nft_id = nft_id;
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
    pub fn finish(self, token_supply: u64) -> Result<NftOutput, Error> {
        let unlock_conditions = UnlockConditions::new(self.unlock_conditions)?;

        verify_unlock_conditions(&unlock_conditions, &self.nft_id)?;

        let features = Features::new(self.features)?;

        verify_allowed_features(&features, NftOutput::ALLOWED_FEATURES)?;

        let immutable_features = Features::new(self.immutable_features)?;

        verify_allowed_features(&immutable_features, NftOutput::ALLOWED_IMMUTABLE_FEATURES)?;

        let mut output = NftOutput {
            amount: 1u64,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            nft_id: self.nft_id,
            unlock_conditions,
            features,
            immutable_features,
        };

        output.amount = match self.amount {
            OutputBuilderAmount::Amount(amount) => amount,
            OutputBuilderAmount::MinimumStorageDeposit(rent_structure) => {
                Output::Nft(output.clone()).rent_cost(&rent_structure)
            }
        };

        verify_output_amount::<true>(&output.amount, &token_supply)?;

        Ok(output)
    }

    /// Finishes the [`NftOutputBuilder`] into an [`Output`].
    pub fn finish_output(self, token_supply: u64) -> Result<Output, Error> {
        Ok(Output::Nft(self.finish(token_supply)?))
    }
}

impl From<&NftOutput> for NftOutputBuilder {
    fn from(output: &NftOutput) -> Self {
        NftOutputBuilder {
            amount: OutputBuilderAmount::Amount(output.amount),
            native_tokens: output.native_tokens.to_vec(),
            nft_id: output.nft_id,
            unlock_conditions: output.unlock_conditions.to_vec(),
            features: output.features.to_vec(),
            immutable_features: output.immutable_features.to_vec(),
        }
    }
}

/// Describes an NFT output, a globally unique token with metadata attached.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NftOutput {
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // Unique identifier of the NFT.
    nft_id: NftId,
    unlock_conditions: UnlockConditions,
    features: Features,
    immutable_features: Features,
}

impl NftOutput {
    /// The [`Output`](crate::output::Output) kind of an [`NftOutput`].
    pub const KIND: u8 = 6;
    /// The set of allowed [`UnlockCondition`]s for an [`NftOutput`].
    pub const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::ADDRESS
        .union(UnlockConditionFlags::STORAGE_DEPOSIT_RETURN)
        .union(UnlockConditionFlags::TIMELOCK)
        .union(UnlockConditionFlags::EXPIRATION);
    /// The set of allowed [`Feature`]s for an [`NftOutput`].
    pub const ALLOWED_FEATURES: FeatureFlags = FeatureFlags::SENDER
        .union(FeatureFlags::METADATA)
        .union(FeatureFlags::TAG);
    /// The set of allowed immutable [`Feature`]s for an [`NftOutput`].
    pub const ALLOWED_IMMUTABLE_FEATURES: FeatureFlags = FeatureFlags::ISSUER.union(FeatureFlags::METADATA);

    /// Creates a new [`NftOutput`] with a provided amount.
    #[inline(always)]
    pub fn new_with_amount(amount: u64, nft_id: NftId, token_supply: u64) -> Result<Self, Error> {
        NftOutputBuilder::new_with_amount(amount, nft_id)?.finish(token_supply)
    }

    /// Creates a new [`NftOutput`] with a provided rent structure.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn new_with_minimum_storage_deposit(
        nft_id: NftId,
        rent_structure: RentStructure,
        token_supply: u64,
    ) -> Result<Self, Error> {
        NftOutputBuilder::new_with_minimum_storage_deposit(rent_structure, nft_id)?.finish(token_supply)
    }

    /// Creates a new [`NftOutputBuilder`] with a provided amount.
    #[inline(always)]
    pub fn build_with_amount(amount: u64, nft_id: NftId) -> Result<NftOutputBuilder, Error> {
        NftOutputBuilder::new_with_amount(amount, nft_id)
    }

    /// Creates a new [`NftOutputBuilder`] with a provided rent structure.
    /// The amount will be set to the minimum storage deposit.
    #[inline(always)]
    pub fn build_with_minimum_storage_deposit(
        rent_structure: RentStructure,
        nft_id: NftId,
    ) -> Result<NftOutputBuilder, Error> {
        NftOutputBuilder::new_with_minimum_storage_deposit(rent_structure, nft_id)
    }

    ///
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    ///
    #[inline(always)]
    pub fn native_tokens(&self) -> &NativeTokens {
        &self.native_tokens
    }

    ///
    #[inline(always)]
    pub fn nft_id(&self) -> &NftId {
        &self.nft_id
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
    pub fn address(&self) -> &Address {
        // An NftOutput must have an AddressUnlockCondition.
        self.unlock_conditions
            .address()
            .map(|unlock_condition| unlock_condition.address())
            .unwrap()
    }

    ///
    #[inline(always)]
    pub fn chain_id(&self) -> ChainId {
        ChainId::Nft(self.nft_id)
    }

    ///
    pub fn unlock(
        &self,
        output_id: &OutputId,
        unlock: &Unlock,
        inputs: &[(OutputId, &Output)],
        context: &mut ValidationContext,
    ) -> Result<(), ConflictReason> {
        self.unlock_conditions()
            .locked_address(self.address(), context.milestone_timestamp)
            .unlock(unlock, inputs, context)?;

        let nft_id = if self.nft_id().is_null() {
            NftId::from(*output_id)
        } else {
            *self.nft_id()
        };

        context
            .unlocked_addresses
            .insert(Address::from(NftAddress::from(nft_id)));

        Ok(())
    }
}

impl StateTransitionVerifier for NftOutput {
    fn creation(next_state: &Self, context: &ValidationContext) -> Result<(), StateTransitionError> {
        if !next_state.nft_id.is_null() {
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
        _context: &ValidationContext,
    ) -> Result<(), StateTransitionError> {
        if current_state.immutable_features != next_state.immutable_features {
            return Err(StateTransitionError::MutatedImmutableField);
        }

        Ok(())
    }

    fn destruction(_current_state: &Self, _context: &ValidationContext) -> Result<(), StateTransitionError> {
        Ok(())
    }
}

impl Packable for NftOutput {
    type UnpackError = Error;
    type UnpackVisitor = ProtocolParameters;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.nft_id.pack(packer)?;
        self.unlock_conditions.pack(packer)?;
        self.features.pack(packer)?;
        self.immutable_features.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let amount = u64::unpack::<_, VERIFY>(unpacker, &()).coerce()?;

        verify_output_amount::<VERIFY>(&amount, &visitor.token_supply()).map_err(UnpackError::Packable)?;

        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker, &())?;
        let nft_id = NftId::unpack::<_, VERIFY>(unpacker, &()).coerce()?;
        let unlock_conditions = UnlockConditions::unpack::<_, VERIFY>(unpacker, visitor)?;

        if VERIFY {
            verify_unlock_conditions(&unlock_conditions, &nft_id).map_err(UnpackError::Packable)?;
        }

        let features = Features::unpack::<_, VERIFY>(unpacker, &())?;

        if VERIFY {
            verify_allowed_features(&features, NftOutput::ALLOWED_FEATURES).map_err(UnpackError::Packable)?;
        }

        let immutable_features = Features::unpack::<_, VERIFY>(unpacker, &())?;

        if VERIFY {
            verify_allowed_features(&immutable_features, NftOutput::ALLOWED_IMMUTABLE_FEATURES)
                .map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            nft_id,
            unlock_conditions,
            features,
            immutable_features,
        })
    }
}

fn verify_unlock_conditions(unlock_conditions: &UnlockConditions, nft_id: &NftId) -> Result<(), Error> {
    if let Some(unlock_condition) = unlock_conditions.address() {
        if let Address::Nft(nft_address) = unlock_condition.address() {
            if nft_address.nft_id() == nft_id {
                return Err(Error::SelfDepositNft(*nft_id));
            }
        }
    } else {
        return Err(Error::MissingAddressUnlockCondition);
    }

    verify_allowed_unlock_conditions(unlock_conditions, NftOutput::ALLOWED_UNLOCK_CONDITIONS)
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{
        error::dto::DtoError,
        output::{
            dto::OutputBuilderAmountDto, feature::dto::FeatureDto, native_token::dto::NativeTokenDto,
            nft_id::dto::NftIdDto, unlock_condition::dto::UnlockConditionDto,
        },
    };

    /// Describes an NFT output, a globally unique token with metadata attached.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct NftOutputDto {
        #[serde(rename = "type")]
        pub kind: u8,
        // Amount of IOTA tokens held by the output.
        pub amount: String,
        // Native tokens held by the output.
        #[serde(rename = "nativeTokens", skip_serializing_if = "Vec::is_empty", default)]
        pub native_tokens: Vec<NativeTokenDto>,
        // Unique identifier of the NFT.
        #[serde(rename = "nftId")]
        pub nft_id: NftIdDto,
        #[serde(rename = "unlockConditions")]
        pub unlock_conditions: Vec<UnlockConditionDto>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub features: Vec<FeatureDto>,
        #[serde(rename = "immutableFeatures", skip_serializing_if = "Vec::is_empty", default)]
        pub immutable_features: Vec<FeatureDto>,
    }

    impl From<&NftOutput> for NftOutputDto {
        fn from(value: &NftOutput) -> Self {
            Self {
                kind: NftOutput::KIND,
                amount: value.amount().to_string(),
                native_tokens: value.native_tokens().iter().map(Into::into).collect::<_>(),
                nft_id: NftIdDto(value.nft_id().to_string()),
                unlock_conditions: value.unlock_conditions().iter().map(Into::into).collect::<_>(),
                features: value.features().iter().map(Into::into).collect::<_>(),
                immutable_features: value.immutable_features().iter().map(Into::into).collect::<_>(),
            }
        }
    }

    impl NftOutput {
        pub fn try_from_dto(value: &NftOutputDto, token_supply: u64) -> Result<NftOutput, DtoError> {
            let mut builder = NftOutputBuilder::new_with_amount(
                value
                    .amount
                    .parse::<u64>()
                    .map_err(|_| DtoError::InvalidField("amount"))?,
                (&value.nft_id).try_into()?,
            )?;

            for t in &value.native_tokens {
                builder = builder.add_native_token(t.try_into()?);
            }

            for u in &value.unlock_conditions {
                builder = builder.add_unlock_condition(UnlockCondition::try_from_dto(u, token_supply)?);
            }

            for b in &value.features {
                builder = builder.add_feature(b.try_into()?);
            }

            for b in &value.immutable_features {
                builder = builder.add_immutable_feature(b.try_into()?);
            }

            Ok(builder.finish(token_supply)?)
        }
    }

    pub fn try_from_dtos(
        amount: OutputBuilderAmountDto,
        native_tokens: Option<Vec<NativeTokenDto>>,
        nft_id: &NftIdDto,
        unlock_conditions: Vec<UnlockConditionDto>,
        features: Option<Vec<FeatureDto>>,
        immutable_features: Option<Vec<FeatureDto>>,
        token_supply: u64,
    ) -> Result<NftOutput, DtoError> {
        let nft_id = NftId::try_from(nft_id)?;

        let mut builder = match amount {
            OutputBuilderAmountDto::Amount(amount) => NftOutputBuilder::new_with_amount(
                amount.parse().map_err(|_| DtoError::InvalidField("amount"))?,
                nft_id,
            )?,
            OutputBuilderAmountDto::MinimumStorageDeposit(rent_structure) => {
                NftOutputBuilder::new_with_minimum_storage_deposit(rent_structure, nft_id)?
            }
        };

        if let Some(native_tokens) = native_tokens {
            let native_tokens = native_tokens
                .iter()
                .map(NativeToken::try_from)
                .collect::<Result<Vec<NativeToken>, DtoError>>()?;
            builder = builder.with_native_tokens(native_tokens);
        }

        let unlock_conditions = unlock_conditions
            .iter()
            .map(|u| UnlockCondition::try_from_dto(u, token_supply))
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

        Ok(builder.finish(token_supply)?)
    }
}
