// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{FeatureBlock, NativeToken},
    Error,
};

use bee_common::packable::{Packable, Read, Write};

///
pub struct ExtendedOutputBuilder {
    amount: u64,
    native_tokens: Vec<NativeToken>,
    address: Address,
    feature_blocks: Vec<FeatureBlock>,
}

impl ExtendedOutputBuilder {
    ///
    pub fn new(amount: u64, address: Address) -> Self {
        Self {
            amount,
            native_tokens: Vec::new(),
            address,
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
    pub fn build(self) -> ExtendedOutput {
        ExtendedOutput {
            amount: self.amount,
            native_tokens: self.native_tokens.into_boxed_slice(),
            address: self.address,
            feature_blocks: self.feature_blocks.into_boxed_slice(),
        }
    }
}

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtendedOutput {
    amount: u64,
    native_tokens: Box<[NativeToken]>,
    address: Address,
    feature_blocks: Box<[FeatureBlock]>,
}

impl ExtendedOutput {
    /// The output kind of an `ExtendedOutput`.
    pub const KIND: u8 = 1;

    /// Creates a new `ExtendedOutput`.
    pub fn new(amount: u64, address: Address) -> Self {
        ExtendedOutputBuilder::new(amount, address).build()
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
    pub fn address(&self) -> &Address {
        &self.address
    }

    ///
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl Packable for ExtendedOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len()
            + 0u16.packed_len()
            + self.native_tokens.iter().map(Packable::packed_len).sum::<usize>()
            + self.address.packed_len()
            + 0u16.packed_len()
            + self.feature_blocks.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;
        (self.native_tokens.len() as u16).pack(writer)?;
        for native_token in self.native_tokens.iter() {
            native_token.pack(writer)?
        }
        self.address.pack(writer)?;
        (self.feature_blocks.len() as u16).pack(writer)?;
        for feature_block in self.feature_blocks.iter() {
            feature_block.pack(writer)?
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let native_tokens_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut native_tokens = Vec::with_capacity(native_tokens_len);
        for _ in 0..native_tokens_len {
            native_tokens.push(NativeToken::unpack_inner::<R, CHECK>(reader)?);
        }
        let feature_blocks_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut feature_blocks = Vec::with_capacity(feature_blocks_len);
        for _ in 0..feature_blocks_len {
            feature_blocks.push(FeatureBlock::unpack_inner::<R, CHECK>(reader)?);
        }
        let address = Address::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            amount,
            native_tokens: native_tokens.into_boxed_slice(),
            address,
            feature_blocks: feature_blocks.into_boxed_slice(),
        })
    }
}
