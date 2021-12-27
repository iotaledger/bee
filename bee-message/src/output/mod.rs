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
pub(crate) use treasury::TreasuryAmount;
pub use treasury::TreasuryOutput;

use crate::Error;

use bee_common::packable::{Packable as OldPackable, Read, Write};

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

impl OldPackable for Output {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Simple(output) => SimpleOutput::KIND.packed_len() + output.packed_len(),
            Self::Treasury(output) => TreasuryOutput::KIND.packed_len() + output.packed_len(),
            Self::Extended(output) => ExtendedOutput::KIND.packed_len() + output.packed_len(),
            Self::Alias(output) => AliasOutput::KIND.packed_len() + output.packed_len(),
            Self::Foundry(output) => FoundryOutput::KIND.packed_len() + output.packed_len(),
            Self::Nft(output) => NftOutput::KIND.packed_len() + output.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Simple(output) => {
                SimpleOutput::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Treasury(output) => {
                TreasuryOutput::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Extended(output) => {
                ExtendedOutput::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Alias(output) => {
                AliasOutput::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Foundry(output) => {
                FoundryOutput::KIND.pack(writer)?;
                output.pack(writer)?;
            }
            Self::Nft(output) => {
                NftOutput::KIND.pack(writer)?;
                output.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            SimpleOutput::KIND => SimpleOutput::unpack_inner::<R, CHECK>(reader)?.into(),
            TreasuryOutput::KIND => TreasuryOutput::unpack_inner::<R, CHECK>(reader)?.into(),
            ExtendedOutput::KIND => ExtendedOutput::unpack_inner::<R, CHECK>(reader)?.into(),
            AliasOutput::KIND => AliasOutput::unpack_inner::<R, CHECK>(reader)?.into(),
            FoundryOutput::KIND => FoundryOutput::unpack_inner::<R, CHECK>(reader)?.into(),
            NftOutput::KIND => NftOutput::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidOutputKind(k)),
        })
    }
}
