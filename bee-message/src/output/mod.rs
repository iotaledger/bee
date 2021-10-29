// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod alias_id;
mod extended;
mod feature_block;
mod foundry;
mod native_token;
mod nft;
mod output_id;
mod simple;
mod token_id;
mod treasury;

pub use alias::{AliasOutput, AliasOutputBuilder};
pub use alias_id::AliasId;
pub use extended::{ExtendedOutput, ExtendedOutputBuilder};
pub use feature_block::FeatureBlock;
pub use foundry::{FoundryOutput, FoundryOutputBuilder, TokenScheme};
pub use native_token::NativeToken;
pub use nft::NftOutput;
pub use output_id::{OutputId, OUTPUT_ID_LENGTH};
pub use simple::{SimpleOutput, SIMPLE_OUTPUT_AMOUNT};
pub use token_id::TokenId;
pub use treasury::{TreasuryOutput, TREASURY_OUTPUT_AMOUNT};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Output {
    /// A simple output.
    Simple(SimpleOutput),
    /// A treasury output.
    Treasury(TreasuryOutput),
    /// An extended output.
    Extended(ExtendedOutput),
    /// An alias output.
    Alias(AliasOutput),
    /// A foundry output.
    Foundry(FoundryOutput),
    /// A NFT output.
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
}

impl Packable for Output {
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
