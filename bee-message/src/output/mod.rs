// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod alias_id;
mod chain_id;
mod extended;
mod foundry;
mod native_token;
mod nft;
mod nft_id;
mod output_id;
mod simple;
mod token_id;
mod treasury;

///
pub mod feature_block;

pub(crate) use alias::StateMetadataLength;
pub use alias::{AliasOutput, AliasOutputBuilder};
pub use alias_id::AliasId;
pub use chain_id::ChainId;
pub use extended::{ExtendedOutput, ExtendedOutputBuilder};
pub(crate) use feature_block::{DustDepositAmount, IndexationFeatureBlockLength, MetadataFeatureBlockLength};
pub use feature_block::{FeatureBlock, FeatureBlocks};
pub use foundry::{FoundryOutput, FoundryOutputBuilder, TokenScheme};
pub(crate) use native_token::NativeTokenCount;
pub use native_token::{NativeToken, NativeTokens};
pub(crate) use nft::ImmutableMetadataLength;
pub use nft::{NftOutput, NftOutputBuilder};
pub use nft_id::NftId;
pub(crate) use output_id::InputOutputIndex;
pub use output_id::OutputId;
pub use simple::SimpleOutput;
pub(crate) use simple::SimpleOutputAmount;
pub use token_id::TokenId;
pub use treasury::TreasuryOutput;
pub(crate) use treasury::TreasuryOutputAmount;

use crate::Error;

use derive_more::From;

use core::ops::RangeInclusive;

/// The maximum number of outputs of a transaction.
pub const OUTPUT_COUNT_MAX: u16 = 127;
/// The range of valid numbers of outputs of a transaction .
pub const OUTPUT_COUNT_RANGE: RangeInclusive<u16> = 1..=OUTPUT_COUNT_MAX; // [1..127]
/// The maximum index of outputs of a transaction.
pub const OUTPUT_INDEX_MAX: u16 = OUTPUT_COUNT_MAX - 1; // 126
/// The range of valid indices of outputs of a transaction .
pub const OUTPUT_INDEX_RANGE: RangeInclusive<u16> = 0..=OUTPUT_INDEX_MAX; // [0..126]

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, bee_packable::Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidOutputKind)]
pub enum Output {
    /// A simple output.
    #[packable(tag = SimpleOutput::KIND)]
    Simple(SimpleOutput),
    /// A treasury output.
    #[packable(tag = TreasuryOutput::KIND)]
    Treasury(TreasuryOutput),
    /// An extended output.
    #[packable(tag = ExtendedOutput::KIND)]
    Extended(ExtendedOutput),
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
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Simple(_) => SimpleOutput::KIND,
            Self::Treasury(_) => TreasuryOutput::KIND,
            Self::Extended(_) => ExtendedOutput::KIND,
            Self::Alias(_) => AliasOutput::KIND,
            Self::Foundry(_) => FoundryOutput::KIND,
            Self::Nft(_) => NftOutput::KIND,
        }
    }

    ///
    pub fn amount(&self) -> u64 {
        match self {
            Self::Simple(output) => output.amount(),
            Self::Treasury(output) => output.amount(),
            Self::Extended(output) => output.amount(),
            Self::Alias(output) => output.amount(),
            Self::Foundry(output) => output.amount(),
            Self::Nft(output) => output.amount(),
        }
    }
}
