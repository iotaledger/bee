// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod alias_id;
mod basic;
mod chain_id;
mod foundry;
mod foundry_id;
mod inputs_commitment;
mod native_token;
mod nft;
mod nft_id;
mod output_id;
mod rent;
mod state_transition;
mod token_id;
mod token_scheme;
mod treasury;

///
pub mod feature;
///
pub mod unlock_condition;

use core::ops::RangeInclusive;

use derive_more::From;
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

pub(crate) use self::{
    alias::StateMetadataLength,
    feature::{MetadataFeatureLength, TagFeatureLength},
    native_token::NativeTokenCount,
    output_id::OutputIndex,
    unlock_condition::AddressUnlockCondition,
};
pub use self::{
    alias::{AliasOutput, AliasOutputBuilder},
    alias_id::AliasId,
    basic::{BasicOutput, BasicOutputBuilder},
    chain_id::ChainId,
    feature::{Feature, Features},
    foundry::{FoundryOutput, FoundryOutputBuilder},
    foundry_id::FoundryId,
    inputs_commitment::InputsCommitment,
    native_token::{NativeToken, NativeTokens, NativeTokensBuilder},
    nft::{NftOutput, NftOutputBuilder},
    nft_id::NftId,
    output_id::OutputId,
    rent::{Rent, RentStructure, RentStructureBuilder},
    state_transition::{StateTransitionError, StateTransitionVerifier},
    token_id::TokenId,
    token_scheme::{SimpleTokenScheme, TokenScheme},
    treasury::TreasuryOutput,
    unlock_condition::{UnlockCondition, UnlockConditions},
};
use crate::{address::Address, protocol::ProtocolParameters, semantic::ValidationContext, Error};

/// The maximum number of outputs of a transaction.
pub const OUTPUT_COUNT_MAX: u16 = 128;
/// The range of valid numbers of outputs of a transaction .
pub const OUTPUT_COUNT_RANGE: RangeInclusive<u16> = 1..=OUTPUT_COUNT_MAX; // [1..128]
/// The maximum index of outputs of a transaction.
pub const OUTPUT_INDEX_MAX: u16 = OUTPUT_COUNT_MAX - 1; // 127
/// The range of valid indices of outputs of a transaction .
pub const OUTPUT_INDEX_RANGE: RangeInclusive<u16> = 0..=OUTPUT_INDEX_MAX; // [0..127]

#[derive(Clone)]
pub(crate) enum OutputBuilderAmount {
    Amount(u64),
    MinimumStorageDeposit(RentStructure),
}

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Output {
    /// A treasury output.
    Treasury(TreasuryOutput),
    /// A basic output.
    Basic(BasicOutput),
    /// An alias output.
    Alias(AliasOutput),
    /// A foundry output.
    Foundry(FoundryOutput),
    /// An NFT output.
    Nft(NftOutput),
}

impl Output {
    /// Minimum amount for an output.
    pub const AMOUNT_MIN: u64 = 1;

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

    /// Returns the features of an [`Output`], if any.
    pub fn features(&self) -> Option<&Features> {
        match self {
            Self::Treasury(_) => None,
            Self::Basic(output) => Some(output.features()),
            Self::Alias(output) => Some(output.features()),
            Self::Foundry(output) => Some(output.features()),
            Self::Nft(output) => Some(output.features()),
        }
    }

    /// Returns the immutable features of an [`Output`], if any.
    pub fn immutable_features(&self) -> Option<&Features> {
        match self {
            Self::Treasury(_) => None,
            Self::Basic(_) => None,
            Self::Alias(output) => Some(output.immutable_features()),
            Self::Foundry(output) => Some(output.immutable_features()),
            Self::Nft(output) => Some(output.immutable_features()),
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
    /// byte cost, given by [`RentStructure`].
    /// If there is a [`StorageDepositReturnUnlockCondition`](unlock_condition::StorageDepositReturnUnlockCondition),
    /// its amount is also checked.
    pub fn verify_storage_deposit(&self, protocol_parameters: &ProtocolParameters) -> Result<(), Error> {
        let required_output_amount = self.rent_cost(protocol_parameters.rent_structure());

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

            let minimum_deposit = minimum_storage_deposit(return_condition.return_address(), protocol_parameters);

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

impl Packable for Output {
    type UnpackError = Error;
    type UnpackVisitor = ProtocolParameters;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            Output::Treasury(output) => {
                TreasuryOutput::KIND.pack(packer)?;
                output.pack(packer)
            }
            Output::Basic(output) => {
                BasicOutput::KIND.pack(packer)?;
                output.pack(packer)
            }
            Output::Alias(output) => {
                AliasOutput::KIND.pack(packer)?;
                output.pack(packer)
            }
            Output::Foundry(output) => {
                FoundryOutput::KIND.pack(packer)?;
                output.pack(packer)
            }
            Output::Nft(output) => {
                NftOutput::KIND.pack(packer)?;
                output.pack(packer)
            }
        }?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Ok(match u8::unpack::<_, VERIFY>(unpacker, &()).coerce()? {
            TreasuryOutput::KIND => Output::from(TreasuryOutput::unpack::<_, VERIFY>(unpacker, visitor).coerce()?),
            BasicOutput::KIND => Output::from(BasicOutput::unpack::<_, VERIFY>(unpacker, visitor).coerce()?),
            AliasOutput::KIND => Output::from(AliasOutput::unpack::<_, VERIFY>(unpacker, visitor).coerce()?),
            FoundryOutput::KIND => Output::from(FoundryOutput::unpack::<_, VERIFY>(unpacker, visitor).coerce()?),
            NftOutput::KIND => Output::from(NftOutput::unpack::<_, VERIFY>(unpacker, visitor).coerce()?),
            k => return Err(Error::InvalidOutputKind(k)).map_err(UnpackError::Packable),
        })
    }
}

impl Rent for Output {
    fn weighted_bytes(&self, rent_structure: &RentStructure) -> u64 {
        self.packed_len() as u64 * rent_structure.v_byte_factor_data as u64
    }
}

pub(crate) fn verify_output_amount<const VERIFY: bool>(
    amount: &u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    if VERIFY && (*amount < Output::AMOUNT_MIN || *amount > protocol_parameters.token_supply()) {
        Err(Error::InvalidOutputAmount(*amount))
    } else {
        Ok(())
    }
}

/// Computes the minimum amount that a storage deposit has to match to allow creating a return [`Output`] back to the
/// sender [`Address`].
fn minimum_storage_deposit(address: &Address, protocol_parameters: &ProtocolParameters) -> u64 {
    let address_condition = UnlockCondition::Address(AddressUnlockCondition::new(*address));
    // PANIC: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    BasicOutputBuilder::new_with_minimum_storage_deposit(protocol_parameters.rent_structure().clone())
        .unwrap()
        .add_unlock_condition(address_condition)
        .finish(protocol_parameters)
        .unwrap()
        .amount()
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    use super::*;
    pub use super::{
        alias::dto::{try_from_alias_output_dto_for_alias_output, AliasOutputDto},
        alias_id::dto::AliasIdDto,
        basic::dto::{try_from_basic_output_dto_for_basic_output, BasicOutputDto},
        foundry::dto::{try_from_foundry_output_dto_for_foundry_output, FoundryOutputDto},
        native_token::dto::NativeTokenDto,
        nft::dto::{try_from_nft_output_dto_for_nft_output, NftOutputDto},
        nft_id::dto::NftIdDto,
        token_id::dto::TokenIdDto,
        token_scheme::dto::{SimpleTokenSchemeDto, TokenSchemeDto},
        treasury::dto::{try_from_treasury_output_dto_for_treasury_output, TreasuryOutputDto},
    };
    use crate::error::dto::DtoError;

    #[derive(Clone, Debug, Deserialize)]
    pub enum OutputBuilderAmountDto {
        Amount(String),
        MinimumStorageDeposit(RentStructure),
    }

    /// Describes all the different output types.
    #[derive(Clone, Debug, Eq, PartialEq)]
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

    pub fn try_from_output_dto_for_output(
        value: &OutputDto,
        protocol_parameters: &ProtocolParameters,
    ) -> Result<Output, DtoError> {
        Ok(match value {
            OutputDto::Treasury(o) => Output::Treasury(try_from_treasury_output_dto_for_treasury_output(
                o,
                protocol_parameters,
            )?),
            OutputDto::Basic(o) => Output::Basic(try_from_basic_output_dto_for_basic_output(o, protocol_parameters)?),
            OutputDto::Alias(o) => Output::Alias(try_from_alias_output_dto_for_alias_output(o, protocol_parameters)?),
            OutputDto::Foundry(o) => {
                Output::Foundry(try_from_foundry_output_dto_for_foundry_output(o, protocol_parameters)?)
            }
            OutputDto::Nft(o) => Output::Nft(try_from_nft_output_dto_for_nft_output(o, protocol_parameters)?),
        })
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

#[cfg(feature = "inx")]
#[allow(missing_docs)]
pub mod inx {
    use super::*;
    use crate::{error::inx::InxError, protocol::ProtocolParameters};

    pub fn try_from_raw_output_for_output(
        value: inx_bindings::proto::RawOutput,
        protocol_parameters: &ProtocolParameters,
    ) -> Result<Output, InxError> {
        Output::unpack_verified(value.data, protocol_parameters).map_err(|e| InxError::InvalidRawBytes(e.to_string()))
    }
}
