// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{FeatureBlock, NativeToken, NativeTokens, NftId},
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
    pub fn new(address: Address, amount: u64, nft_id: NftId, immutable_metadata: Vec<u8>) -> Self {
        Self {
            address,
            amount,
            native_tokens: Vec::new(),
            nft_id,
            immutable_metadata,
            feature_blocks: Vec::new(),
        }
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
        Ok(NftOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            nft_id: self.nft_id,
            immutable_metadata: self.immutable_metadata.into_boxed_slice(),
            feature_blocks: self.feature_blocks.into_boxed_slice(),
        })
    }
}

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct NftOutput {
    address: Address,
    amount: u64,
    native_tokens: NativeTokens,
    nft_id: NftId,
    immutable_metadata: Box<[u8]>,
    feature_blocks: Box<[FeatureBlock]>,
}

impl NftOutput {
    /// The output kind of a `NftOutput`.
    pub const KIND: u8 = 5;

    /// Creates a new `NftOutput`.
    pub fn new(address: Address, amount: u64, nft_id: NftId, immutable_metadata: Vec<u8>) -> Self {
        // SAFETY: this can't fail as this is a default builder.
        NftOutputBuilder::new(address, amount, nft_id, immutable_metadata)
            .finish()
            .unwrap()
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
            + 0u16.packed_len()
            + self.feature_blocks.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        self.amount.pack(writer)?;
        self.native_tokens.pack(writer)?;
        self.nft_id.pack(writer)?;
        (self.immutable_metadata.len() as u32).pack(writer)?;
        writer.write_all(&self.immutable_metadata)?;
        (self.feature_blocks.len() as u16).pack(writer)?;
        for feature_block in self.feature_blocks.iter() {
            feature_block.pack(writer)?
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let address = Address::unpack_inner::<R, CHECK>(reader)?;
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let native_tokens = NativeTokens::unpack_inner::<R, CHECK>(reader)?;
        let nft_id = NftId::unpack_inner::<R, CHECK>(reader)?;
        let immutable_metadata_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut immutable_metadata = vec![0u8; immutable_metadata_len];
        reader.read_exact(&mut immutable_metadata)?;
        let feature_blocks_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut feature_blocks = Vec::with_capacity(feature_blocks_len);
        for _ in 0..feature_blocks_len {
            feature_blocks.push(FeatureBlock::unpack_inner::<R, CHECK>(reader)?);
        }

        Ok(Self {
            address,
            amount,
            native_tokens,
            nft_id,
            immutable_metadata: immutable_metadata.into_boxed_slice(),
            feature_blocks: feature_blocks.into_boxed_slice(),
        })
    }
}
