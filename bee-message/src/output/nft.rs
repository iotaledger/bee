// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{
            validate_allowed_feature_blocks, DustDepositReturnFeatureBlock, ExpirationMilestoneIndexFeatureBlock,
            ExpirationUnixFeatureBlock, FeatureBlock, FeatureBlocks, IndexationFeatureBlock, IssuerFeatureBlock,
            MetadataFeatureBlock, SenderFeatureBlock, TimelockMilestoneIndexFeatureBlock, TimelockUnixFeatureBlock,
        },
        NativeToken, NativeTokens, NftId,
    },
    Error,
};

use bee_common::packable::{Packable, Read, Write};

///
pub struct NftOutputBuilder {
    address: Address,
    amount: u64,
    native_tokens: Vec<NativeToken>,
    nft_id: NftId,
    immutable_metadata: Vec<u8>,
    feature_blocks: Vec<FeatureBlock>,
}

impl NftOutputBuilder {
    ///
    pub fn new(
        address: Address,
        amount: u64,
        nft_id: NftId,
        immutable_metadata: Vec<u8>,
    ) -> Result<NftOutputBuilder, Error> {
        validate_address(&address, &nft_id)?;
        validate_immutable_metadata_length(immutable_metadata.len())?;

        Ok(Self {
            address,
            amount,
            native_tokens: Vec::new(),
            nft_id,
            immutable_metadata,
            feature_blocks: Vec::new(),
        })
    }

    ///
    pub fn add_native_token(mut self, native_token: NativeToken) -> Self {
        self.native_tokens.push(native_token);
        self
    }

    ///
    pub fn with_native_tokens(mut self, native_tokens: Vec<NativeToken>) -> Self {
        self.native_tokens = native_tokens;
        self
    }

    ///
    pub fn add_feature_block(mut self, feature_block: FeatureBlock) -> Self {
        self.feature_blocks.push(feature_block);
        self
    }

    ///
    pub fn with_feature_blocks(mut self, feature_blocks: Vec<FeatureBlock>) -> Self {
        self.feature_blocks = feature_blocks;
        self
    }

    ///
    pub fn finish(self) -> Result<NftOutput, Error> {
        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        validate_allowed_feature_blocks(&feature_blocks, &NftOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(NftOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            nft_id: self.nft_id,
            immutable_metadata: self.immutable_metadata.into_boxed_slice(),
            feature_blocks,
        })
    }
}

/// Describes an NFT output, a globally unique token with metadata attached.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct NftOutput {
    // Deposit address of the output.
    address: Address,
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // Unique identifier of the NFT.
    nft_id: NftId,
    // Binary metadata attached immutably to the NFT.
    immutable_metadata: Box<[u8]>,
    feature_blocks: FeatureBlocks,
}

impl NftOutput {
    /// The [`Output`](crate::output::Output) kind of a [`NftOutput`].
    pub const KIND: u8 = 6;
    ///
    pub const IMMUTABLE_METADATA_LENGTH_MAX: usize = 1024;

    ///
    const ALLOWED_FEATURE_BLOCKS: [u8; 9] = [
        SenderFeatureBlock::KIND,
        IssuerFeatureBlock::KIND,
        DustDepositReturnFeatureBlock::KIND,
        TimelockMilestoneIndexFeatureBlock::KIND,
        TimelockUnixFeatureBlock::KIND,
        ExpirationMilestoneIndexFeatureBlock::KIND,
        ExpirationUnixFeatureBlock::KIND,
        MetadataFeatureBlock::KIND,
        IndexationFeatureBlock::KIND,
    ];

    /// Creates a new [`NftOutput`].
    pub fn new(address: Address, amount: u64, nft_id: NftId, immutable_metadata: Vec<u8>) -> Result<Self, Error> {
        NftOutputBuilder::new(address, amount, nft_id, immutable_metadata)?.finish()
    }

    /// Creates a new [`NftOutputBuilder`].
    pub fn build(
        address: Address,
        amount: u64,
        nft_id: NftId,
        immutable_metadata: Vec<u8>,
    ) -> Result<NftOutputBuilder, Error> {
        NftOutputBuilder::new(address, amount, nft_id, immutable_metadata)
    }

    ///
    pub fn address(&self) -> &Address {
        &self.address
    }

    ///
    pub fn amount(&self) -> u64 {
        self.amount
    }

    ///
    pub fn native_tokens(&self) -> &[NativeToken] {
        &self.native_tokens
    }

    ///
    pub fn nft_id(&self) -> &NftId {
        &self.nft_id
    }

    ///
    pub fn immutable_metadata(&self) -> &[u8] {
        &self.immutable_metadata
    }

    ///
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl Packable for NftOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.address.packed_len()
            + self.amount.packed_len()
            + self.native_tokens.packed_len()
            + self.nft_id.packed_len()
            + 0u32.packed_len()
            + self.immutable_metadata.len()
            + self.feature_blocks.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        self.amount.pack(writer)?;
        self.native_tokens.pack(writer)?;
        self.nft_id.pack(writer)?;
        (self.immutable_metadata.len() as u32).pack(writer)?;
        writer.write_all(&self.immutable_metadata)?;
        self.feature_blocks.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let address = Address::unpack_inner::<R, CHECK>(reader)?;
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let native_tokens = NativeTokens::unpack_inner::<R, CHECK>(reader)?;
        let nft_id = NftId::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_address(&address, &nft_id)?;
        }

        let immutable_metadata_length = u32::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK {
            validate_immutable_metadata_length(immutable_metadata_length)?;
        }

        let mut immutable_metadata = vec![0u8; immutable_metadata_length];
        reader.read_exact(&mut immutable_metadata)?;
        let feature_blocks = FeatureBlocks::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_allowed_feature_blocks(&feature_blocks, &NftOutput::ALLOWED_FEATURE_BLOCKS)?;
        }

        Ok(Self {
            address,
            amount,
            native_tokens,
            nft_id,
            immutable_metadata: immutable_metadata.into_boxed_slice(),
            feature_blocks,
        })
    }
}

#[inline]
fn validate_address(address: &Address, nft_id: &NftId) -> Result<(), Error> {
    match address {
        Address::Ed25519(_) => {}
        Address::Alias(_) => {}
        Address::Nft(address) => {
            if address.id() == nft_id {
                return Err(Error::SelfDepositNft(*nft_id));
            }
        }
    };

    Ok(())
}

#[inline]
fn validate_immutable_metadata_length(immutable_metadata_length: usize) -> Result<(), Error> {
    if immutable_metadata_length > NftOutput::IMMUTABLE_METADATA_LENGTH_MAX {
        return Err(Error::InvalidMetadataLength(immutable_metadata_length));
    }

    Ok(())
}
