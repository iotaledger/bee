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
pub use byte_cost::{ByteCost, ByteCostConfig, ByteCostConfigBuilder};
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
pub use unlock_condition::{AddressUnlockCondition, UnlockCondition, UnlockConditions};

use crate::{address::Address, constant::IOTA_SUPPLY, Error};

use derive_more::From;
use packable::{bounded::BoundedU64, PackableExt};

use core::ops::RangeInclusive;

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

    /// Verify if a valid storage deposit was made. Each [`Output`] has to have an amount that covers its associated
    /// byte cost, given by [`ByteCostConfig`]. If there is a
    /// [`StorageDepositReturnUnlockCondition`](unlock_condition::StorageDepositReturnUnlockCondition), its amount
    /// is also checked.
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
            // `0` ≤ `Amount` - `Return Amount`
            if return_condition.amount() > self.amount() {
                return Err(Error::StorageDepositReturnExceedsOutputAmount {
                    deposit: return_condition.amount(),
                    amount: self.amount(),
                });
            }

            let minimum_deposit = minimum_storage_deposit(config, return_condition.return_address());

            // `Return Amount` must be ≥ than `Minimum Storage Deposit`
            if return_condition.amount() < minimum_deposit {
                return Err(Error::InsufficientStorageDepositReturnAmount {
                    deposit: return_condition.amount(),
                    required: minimum_deposit,
                });
            }

            // Check if the storage deposit return was required in the first place.
            // `Amount` - `Return Amount` ≤ `Required Storage Deposit of the Output`
            if self.amount() - return_condition.amount() > required_output_amount {
                return Err(Error::UnnecessaryStorageDepositReturnCondition {
                    logical_amount: self.amount() - return_condition.amount(),
                    required: required_output_amount,
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
    // Safety: This can never fail because the amount will always be within the valid range. Also, the actual value is
    // not important, we are only interested in the storage requirements of the type.
    let basic_output = BasicOutputBuilder::new(OutputAmount::MIN)
        .unwrap()
        .add_unlock_condition(address_condition)
        .finish()
        .unwrap();
    Output::Basic(basic_output).byte_cost(config)
}
