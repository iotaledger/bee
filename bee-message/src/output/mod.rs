// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod alias_id;
mod basic;
mod byte_cost;
mod chain_id;
mod foundry;
mod foundry_id;
mod native_token;
mod nft;
mod nft_id;
mod output_id;
mod state_transition;
mod token_id;
mod token_scheme;
mod treasury;

///
pub mod feature_block;
///
pub mod unlock_condition;

use core::ops::RangeInclusive;

use crypto::hashes::{blake2b::Blake2b256, Digest};
use derive_more::From;
use packable::{bounded::BoundedU64, PackableExt};

pub(crate) use self::{
    alias::StateMetadataLength,
    feature_block::{MetadataFeatureBlockLength, TagFeatureBlockLength},
    native_token::NativeTokenCount,
    output_id::OutputIndex,
    treasury::TreasuryOutputAmount,
    unlock_condition::{AddressUnlockCondition, StorageDepositAmount},
};
pub use self::{
    alias::{AliasOutput, AliasOutputBuilder},
    alias_id::AliasId,
    basic::{BasicOutput, BasicOutputBuilder},
    byte_cost::{ByteCost, ByteCostConfig, ByteCostConfigBuilder},
    chain_id::ChainId,
    feature_block::{FeatureBlock, FeatureBlocks},
    foundry::{FoundryOutput, FoundryOutputBuilder},
    foundry_id::FoundryId,
    native_token::{NativeToken, NativeTokens, NativeTokensBuilder},
    nft::{NftOutput, NftOutputBuilder},
    nft_id::NftId,
    output_id::OutputId,
    state_transition::{StateTransitionError, StateTransitionVerifier},
    token_id::{TokenId, TokenTag},
    token_scheme::{SimpleTokenScheme, TokenScheme},
    treasury::TreasuryOutput,
    unlock_condition::{UnlockCondition, UnlockConditions},
};
use crate::{address::Address, constant::IOTA_SUPPLY, semantic::ValidationContext, Error};

/// The maximum number of outputs of a transaction.
pub const OUTPUT_COUNT_MAX: u16 = 128;
/// The range of valid numbers of outputs of a transaction .
pub const OUTPUT_COUNT_RANGE: RangeInclusive<u16> = 1..=OUTPUT_COUNT_MAX; // [1..128]
/// The maximum index of outputs of a transaction.
pub const OUTPUT_INDEX_MAX: u16 = OUTPUT_COUNT_MAX - 1; // 127
/// The range of valid indices of outputs of a transaction .
pub const OUTPUT_INDEX_RANGE: RangeInclusive<u16> = 0..=OUTPUT_INDEX_MAX; // [0..127]

/// Type representing an output amount.
pub type OutputAmount = BoundedU64<{ *Output::AMOUNT_RANGE.start() }, { *Output::AMOUNT_RANGE.end() }>;

pub(crate) enum OutputBuilderAmount {
    Amount(OutputAmount),
    MinimumStorageDeposit(ByteCostConfig),
}

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidOutputKind)]
pub enum Output {
    /// A treasury output.
    #[packable(tag = TreasuryOutput::KIND)]
    Treasury(TreasuryOutput),
    /// A basic output.
    #[packable(tag = BasicOutput::KIND)]
    Basic(BasicOutput),
    /// An alias output.
    #[packable(tag = AliasOutput::KIND)]
    Alias(AliasOutput),
    /// A foundry output.
    #[packable(tag = FoundryOutput::KIND)]
    Foundry(FoundryOutput),
    /// An NFT output.
    #[packable(tag = NftOutput::KIND)]
    Nft(NftOutput),
}

impl Output {
    /// Valid amounts for an [`Output`].
    pub const AMOUNT_RANGE: RangeInclusive<u64> = 1..=IOTA_SUPPLY;

    /// Return the output kind of an [`Output`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Treasury(_) => TreasuryOutput::KIND,
            Self::Basic(_) => BasicOutput::KIND,
            Self::Alias(_) => AliasOutput::KIND,
            Self::Foundry(_) => FoundryOutput::KIND,
            Self::Nft(_) => NftOutput::KIND,
        }
    }

    /// Returns the amount of an [`Output`].
    pub fn amount(&self) -> u64 {
        match self {
            Self::Treasury(output) => output.amount(),
            Self::Basic(output) => output.amount(),
            Self::Alias(output) => output.amount(),
            Self::Foundry(output) => output.amount(),
            Self::Nft(output) => output.amount(),
        }
    }

    /// Returns the native tokens of an [`Output`], if any.
    pub fn native_tokens(&self) -> Option<&NativeTokens> {
        match self {
            Self::Treasury(_) => None,
            Self::Basic(output) => Some(output.native_tokens()),
            Self::Alias(output) => Some(output.native_tokens()),
            Self::Foundry(output) => Some(output.native_tokens()),
            Self::Nft(output) => Some(output.native_tokens()),
        }
    }

    /// Returns the unlock conditions of an [`Output`], if any.
    pub fn unlock_conditions(&self) -> Option<&UnlockConditions> {
        match self {
            Self::Treasury(_) => None,
            Self::Basic(output) => Some(output.unlock_conditions()),
            Self::Alias(output) => Some(output.unlock_conditions()),
            Self::Foundry(output) => Some(output.unlock_conditions()),
            Self::Nft(output) => Some(output.unlock_conditions()),
        }
    }

    /// Returns the feature blocks of an [`Output`], if any.
    pub fn feature_blocks(&self) -> Option<&FeatureBlocks> {
        match self {
            Self::Treasury(_) => None,
            Self::Basic(output) => Some(output.feature_blocks()),
            Self::Alias(output) => Some(output.feature_blocks()),
            Self::Foundry(output) => Some(output.feature_blocks()),
            Self::Nft(output) => Some(output.feature_blocks()),
        }
    }

    /// Returns the immutable feature blocks of an [`Output`], if any.
    pub fn immutable_feature_blocks(&self) -> Option<&FeatureBlocks> {
        match self {
            Self::Treasury(_) => None,
            Self::Basic(_) => None,
            Self::Alias(output) => Some(output.immutable_feature_blocks()),
            Self::Foundry(output) => Some(output.immutable_feature_blocks()),
            Self::Nft(output) => Some(output.immutable_feature_blocks()),
        }
    }

    /// Returns the chain identifier of an [`Output`], if any.
    pub fn chain_id(&self) -> Option<ChainId> {
        match self {
            Self::Treasury(_) => None,
            Self::Basic(_) => None,
            Self::Alias(output) => Some(output.chain_id()),
            Self::Foundry(output) => Some(output.chain_id()),
            Self::Nft(output) => Some(output.chain_id()),
        }
    }

    ///
    pub fn verify_state_transition(
        current_state: Option<&Output>,
        next_state: Option<&Output>,
        context: &ValidationContext,
    ) -> Result<(), StateTransitionError> {
        match (current_state, next_state) {
            // Creations.
            (None, Some(Output::Alias(next_state))) => AliasOutput::creation(next_state, context),
            (None, Some(Output::Foundry(next_state))) => FoundryOutput::creation(next_state, context),
            (None, Some(Output::Nft(next_state))) => NftOutput::creation(next_state, context),

            // Transitions.
            (Some(Output::Alias(current_state)), Some(Output::Alias(next_state))) => {
                AliasOutput::transition(current_state, next_state, context)
            }
            (Some(Output::Foundry(current_state)), Some(Output::Foundry(next_state))) => {
                FoundryOutput::transition(current_state, next_state, context)
            }
            (Some(Output::Nft(current_state)), Some(Output::Nft(next_state))) => {
                NftOutput::transition(current_state, next_state, context)
            }

            // Destructions.
            (Some(Output::Alias(current_state)), None) => AliasOutput::destruction(current_state, context),
            (Some(Output::Foundry(current_state)), None) => FoundryOutput::destruction(current_state, context),
            (Some(Output::Nft(current_state)), None) => NftOutput::destruction(current_state, context),

            // Unsupported.
            _ => Err(StateTransitionError::UnsupportedStateTransition),
        }
    }

    /// Verifies if a valid storage deposit was made. Each [`Output`] has to have an amount that covers its associated
    /// byte cost, given by [`ByteCostConfig`].
    /// If there is a [`StorageDepositReturnUnlockCondition`](unlock_condition::StorageDepositReturnUnlockCondition),
    /// its amount is also checked.
    pub fn verify_storage_deposit(&self, config: &ByteCostConfig) -> Result<(), Error> {
        let required_output_amount = self.byte_cost(config);

        if self.amount() < required_output_amount {
            return Err(Error::InsufficientStorageDepositAmount {
                amount: self.amount(),
                required: required_output_amount,
            });
        }

        if let Some(return_condition) = self
            .unlock_conditions()
            .and_then(UnlockConditions::storage_deposit_return)
        {
            // We can't return more tokens than were originally contained in the output.
            // `Return Amount` ≤ `Amount`.
            if return_condition.amount() > self.amount() {
                return Err(Error::StorageDepositReturnExceedsOutputAmount {
                    deposit: return_condition.amount(),
                    amount: self.amount(),
                });
            }

            let minimum_deposit = minimum_storage_deposit(config, return_condition.return_address());

            // `Minimum Storage Deposit` ≤  `Return Amount`
            if return_condition.amount() < minimum_deposit {
                return Err(Error::InsufficientStorageDepositReturnAmount {
                    deposit: return_condition.amount(),
                    required: minimum_deposit,
                });
            }
        }

        Ok(())
    }
}

impl ByteCost for Output {
    fn weighted_bytes(&self, config: &ByteCostConfig) -> u64 {
        self.packed_len() as u64 * config.v_byte_factor_data
    }
}

/// Computes the minimum amount that a storage deposit has to match to allow creating a return [`Output`] back to the
/// sender [`Address`].
fn minimum_storage_deposit(config: &ByteCostConfig, address: &Address) -> u64 {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // PANIC: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    BasicOutputBuilder::new_with_minimum_storage_deposit(config.clone())
        .unwrap()
        .add_unlock_condition(address_condition)
        .finish()
        .unwrap()
        .amount()
}

///
pub fn create_inputs_commitment<'a>(inputs: impl Iterator<Item = &'a Output>) -> [u8; 32] {
    let mut hasher = Blake2b256::new();

    inputs.for_each(|output| hasher.update(output.pack_to_vec()));

    hasher.finalize().into()
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    use super::*;
    pub use super::{
        alias::dto::AliasOutputDto, basic::dto::BasicOutputDto, foundry::dto::FoundryOutputDto, nft::dto::NftOutputDto,
        treasury::dto::TreasuryOutputDto,
    };
    use crate::error::dto::DtoError;

    /// Describes all the different output types.
    #[derive(Clone, Debug)]
    pub enum OutputDto {
        Treasury(TreasuryOutputDto),
        Basic(BasicOutputDto),
        Alias(AliasOutputDto),
        Foundry(FoundryOutputDto),
        Nft(NftOutputDto),
    }

    impl From<&Output> for OutputDto {
        fn from(value: &Output) -> Self {
            match value {
                Output::Treasury(o) => OutputDto::Treasury(o.into()),
                Output::Basic(o) => OutputDto::Basic(o.into()),
                Output::Alias(o) => OutputDto::Alias(o.into()),
                Output::Foundry(o) => OutputDto::Foundry(o.into()),
                Output::Nft(o) => OutputDto::Nft(o.into()),
            }
        }
    }

    impl TryFrom<&OutputDto> for Output {
        type Error = DtoError;

        fn try_from(value: &OutputDto) -> Result<Self, Self::Error> {
            match value {
                OutputDto::Treasury(o) => Ok(Output::Treasury(o.try_into()?)),
                OutputDto::Basic(o) => Ok(Output::Basic(o.try_into()?)),
                OutputDto::Alias(o) => Ok(Output::Alias(o.try_into()?)),
                OutputDto::Foundry(o) => Ok(Output::Foundry(o.try_into()?)),
                OutputDto::Nft(o) => Ok(Output::Nft(o.try_into()?)),
            }
        }
    }

    impl<'de> Deserialize<'de> for OutputDto {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;
            Ok(
                match value
                    .get("type")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| serde::de::Error::custom("invalid output type"))? as u8
                {
                    TreasuryOutput::KIND => {
                        OutputDto::Treasury(TreasuryOutputDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize treasury output: {}", e))
                        })?)
                    }
                    BasicOutput::KIND => OutputDto::Basic(
                        BasicOutputDto::deserialize(value)
                            .map_err(|e| serde::de::Error::custom(format!("cannot deserialize basic output: {}", e)))?,
                    ),
                    AliasOutput::KIND => OutputDto::Alias(
                        AliasOutputDto::deserialize(value)
                            .map_err(|e| serde::de::Error::custom(format!("cannot deserialize alias output: {}", e)))?,
                    ),
                    FoundryOutput::KIND => {
                        OutputDto::Foundry(FoundryOutputDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize foundry output: {}", e))
                        })?)
                    }
                    NftOutput::KIND => OutputDto::Nft(
                        NftOutputDto::deserialize(value)
                            .map_err(|e| serde::de::Error::custom(format!("cannot deserialize NFT output: {}", e)))?,
                    ),
                    _ => return Err(serde::de::Error::custom("invalid output type")),
                },
            )
        }
    }

    impl Serialize for OutputDto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum OutputDto_<'a> {
                T1(&'a TreasuryOutputDto),
                T2(&'a BasicOutputDto),
                T3(&'a AliasOutputDto),
                T4(&'a FoundryOutputDto),
                T5(&'a NftOutputDto),
            }
            #[derive(Serialize)]
            struct TypedOutput<'a> {
                #[serde(flatten)]
                output: OutputDto_<'a>,
            }
            let output = match self {
                OutputDto::Treasury(o) => TypedOutput {
                    output: OutputDto_::T1(o),
                },
                OutputDto::Basic(o) => TypedOutput {
                    output: OutputDto_::T2(o),
                },
                OutputDto::Alias(o) => TypedOutput {
                    output: OutputDto_::T3(o),
                },
                OutputDto::Foundry(o) => TypedOutput {
                    output: OutputDto_::T4(o),
                },
                OutputDto::Nft(o) => TypedOutput {
                    output: OutputDto_::T5(o),
                },
            };
            output.serialize(serializer)
        }
    }
}
