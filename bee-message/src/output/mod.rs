// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod alias_id;
mod basic;
mod byte_cost;
mod chain_id;
#[cfg(feature = "cpt2")]
mod cpt2;
mod foundry;
mod foundry_id;
mod native_token;
mod nft;
mod nft_id;
mod output_id;
mod token_id;
mod token_scheme;
mod treasury;

///
pub mod feature_block;
///
pub mod unlock_condition;

pub(crate) use alias::StateMetadataLength;
pub use alias::{AliasOutput, AliasOutputBuilder};
pub use alias_id::AliasId;
pub use basic::{BasicOutput, BasicOutputBuilder};
pub(crate) use byte_cost::{minimum_storage_deposit, ByteCost};
pub use byte_cost::{ByteCostConfig, ByteCostConfigBuilder};
pub use chain_id::ChainId;
#[cfg(feature = "cpt2")]
pub(crate) use cpt2::signature_locked_dust_allowance::DustAllowanceAmount;
#[cfg(feature = "cpt2")]
pub use cpt2::{
    signature_locked_dust_allowance::{dust_outputs_max, SignatureLockedDustAllowanceOutput},
    signature_locked_single::SignatureLockedSingleOutput,
};
pub use feature_block::{FeatureBlock, FeatureBlocks};
pub(crate) use feature_block::{MetadataFeatureBlockLength, TagFeatureBlockLength};
pub use foundry::{FoundryOutput, FoundryOutputBuilder};
pub use foundry_id::FoundryId;
pub(crate) use native_token::NativeTokenCount;
pub use native_token::{NativeToken, NativeTokens};
pub use nft::{NftOutput, NftOutputBuilder};
pub use nft_id::NftId;
pub use output_id::OutputId;
pub(crate) use output_id::OutputIndex;
pub use token_id::{TokenId, TokenTag};
pub use token_scheme::TokenScheme;
pub use treasury::TreasuryOutput;
pub(crate) use treasury::TreasuryOutputAmount;
pub(crate) use unlock_condition::StorageDepositAmount;
pub use unlock_condition::{UnlockCondition, UnlockConditions};

use crate::{constant::IOTA_SUPPLY, Error};

use derive_more::From;
use packable::{bounded::BoundedU64, PackableExt};

use core::ops::RangeInclusive;

use self::unlock_condition::StorageDepositReturnUnlockCondition;

/// The maximum number of outputs of a transaction.
pub const OUTPUT_COUNT_MAX: u16 = 128;
/// The range of valid numbers of outputs of a transaction .
pub const OUTPUT_COUNT_RANGE: RangeInclusive<u16> = 1..=OUTPUT_COUNT_MAX; // [1..128]
/// The maximum index of outputs of a transaction.
pub const OUTPUT_INDEX_MAX: u16 = OUTPUT_COUNT_MAX - 1; // 127
/// The range of valid indices of outputs of a transaction .
pub const OUTPUT_INDEX_RANGE: RangeInclusive<u16> = 0..=OUTPUT_INDEX_MAX; // [0..127]

pub(crate) type OutputAmount = BoundedU64<{ *Output::AMOUNT_RANGE.start() }, { *Output::AMOUNT_RANGE.end() }>;

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidOutputKind)]
pub enum Output {
    /// A chrysalis pt2 signature locked single output.
    #[cfg(feature = "cpt2")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
    #[packable(tag = SignatureLockedSingleOutput::KIND)]
    SignatureLockedSingle(SignatureLockedSingleOutput),
    /// A chrysalis pt2 signature locked dust allowance output.
    #[cfg(feature = "cpt2")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "cpt2")))]
    #[packable(tag = SignatureLockedDustAllowanceOutput::KIND)]
    SignatureLockedDustAllowance(SignatureLockedDustAllowanceOutput),
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
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedSingle(_) => SignatureLockedSingleOutput::KIND,
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedDustAllowance(_) => SignatureLockedDustAllowanceOutput::KIND,
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
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedSingle(output) => output.amount(),
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedDustAllowance(output) => output.amount(),
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
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedSingle(_) => None,
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedDustAllowance(_) => None,
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
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedSingle(_) => None,
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedDustAllowance(_) => None,
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
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedSingle(_) => None,
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedDustAllowance(_) => None,
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
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedSingle(_) => None,
            #[cfg(feature = "cpt2")]
            Self::SignatureLockedDustAllowance(_) => None,
            Self::Treasury(_) => None,
            Self::Basic(_) => None,
            Self::Alias(output) => Some(output.immutable_feature_blocks()),
            Self::Foundry(output) => Some(output.immutable_feature_blocks()),
            Self::Nft(output) => Some(output.immutable_feature_blocks()),
        }
    }

    /// Checks if a sufficient storage deposit was made for the given [`Output`].
    pub fn check_sufficient_storage_deposit(&self, config: &ByteCostConfig) -> Result<(), Error> {
        let maybe_deposit_condition = match self {
            #[cfg(feature = "cpt2")]
            Output::SignatureLockedSingle(_) | Output::SignatureLockedDustAllowance(_) => return Ok(()),
            Output::Treasury(_) => return Ok(()), // `TreasureOutput`s don't have `UnlockConditions`.
            Output::Basic(_) | Output::Alias(_) | Output::Foundry(_) | Output::Nft(_) => self
                .unlock_conditions()
                .map(|conds| conds.get(StorageDepositReturnUnlockCondition::KIND))
                .flatten(),
        };

        let required = minimum_storage_deposit(config, self);

        let maybe_deposit =
            if let Some(UnlockCondition::StorageDepositReturn(storage_deposit_condition)) = maybe_deposit_condition {
                Some(storage_deposit_condition.amount())
            } else {
                None
            };

        if self.amount() >= required {
            match maybe_deposit {
                Some(deposit) if deposit >= required => Ok(()),
                Some(deposit) => Err(Error::InsufficientStorageDepositReturnAmount { required, deposit }),
                None => Ok(()),
            }
        } else {
            match maybe_deposit {
                Some(deposit) if deposit >= required => Ok(()),
                _ => Err(Error::InsufficientStorageDepositAmount {
                    required,
                    amount: self.amount(),
                }),
            }
        }
    }
}

impl ByteCost for Output {
    fn weighted_bytes(&self, config: &ByteCostConfig) -> u64 {
        self.packed_len() as u64 * config.v_byte_factor_data
    }
}
