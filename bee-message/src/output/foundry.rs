// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{FeatureBlock, NativeToken},
    Error,
};

use bee_common::packable::{Packable, Read, Write};

use primitive_types::U256;

///
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub enum TokenScheme {
    ///
    Simple = 0,
}

impl Packable for TokenScheme {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (*self as u8).pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            0 => TokenScheme::Simple,
            k => return Err(Self::Error::InvalidTokenSchemeKind(k)),
        })
    }
}

///
pub struct FoundryOutputBuilder {
    address: Address,
    amount: u64,
    native_tokens: Vec<NativeToken>,
    serial_number: u32,
    token_tag: [u8; 12],
    circulating_supply: U256,
    maximum_supply: U256,
    token_scheme: TokenScheme,
    feature_blocks: Vec<FeatureBlock>,
}

impl FoundryOutputBuilder {
    ///
    pub fn new(
        address: Address,
        amount: u64,
        serial_number: u32,
        token_tag: [u8; 12],
        circulating_supply: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Self {
        Self {
            address,
            amount,
            native_tokens: Vec::new(),
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
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
    pub fn finish(self) -> FoundryOutput {
        FoundryOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: self.native_tokens.into_boxed_slice(),
            serial_number: self.serial_number,
            token_tag: self.token_tag,
            circulating_supply: self.circulating_supply,
            maximum_supply: self.maximum_supply,
            token_scheme: self.token_scheme,
            feature_blocks: self.feature_blocks.into_boxed_slice(),
        }
    }
}

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FoundryOutput {
    address: Address,
    amount: u64,
    native_tokens: Box<[NativeToken]>,
    serial_number: u32,
    token_tag: [u8; 12],
    circulating_supply: U256,
    maximum_supply: U256,
    token_scheme: TokenScheme,
    feature_blocks: Box<[FeatureBlock]>,
}

impl FoundryOutput {
    /// The output kind of a `FoundryOutput`.
    pub const KIND: u8 = 4;

    /// Creates a new `FoundryOutput`.
    pub fn new(
        address: Address,
        amount: u64,
        serial_number: u32,
        token_tag: [u8; 12],
        circulating_supply: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Self {
        FoundryOutputBuilder::new(
            address,
            amount,
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
        )
        .finish()
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
    pub fn serial_number(&self) -> u32 {
        self.serial_number
    }

    ///
    pub fn token_tag(&self) -> &[u8; 12] {
        &self.token_tag
    }

    ///
    pub fn circulating_supply(&self) -> &U256 {
        &self.circulating_supply
    }

    ///
    pub fn maximum_supply(&self) -> &U256 {
        &self.maximum_supply
    }

    ///
    pub fn token_scheme(&self) -> TokenScheme {
        self.token_scheme
    }

    ///
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl Packable for FoundryOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.address.packed_len()
            + self.amount.packed_len()
            + 0u16.packed_len()
            + self.native_tokens.iter().map(Packable::packed_len).sum::<usize>()
            + self.serial_number.packed_len()
            + self.token_tag.packed_len()
            + 32
            + 32
            + self.token_scheme.packed_len()
            + 0u16.packed_len()
            + self.feature_blocks.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        self.amount.pack(writer)?;
        (self.native_tokens.len() as u16).pack(writer)?;
        for native_token in self.native_tokens.iter() {
            native_token.pack(writer)?
        }
        self.serial_number.pack(writer)?;
        self.token_tag.pack(writer)?;
        // SAFETY: Reinterpreting a [u64; 4] as a [u8; 32] is fine since they have the same size.
        writer.write_all(&unsafe { std::mem::transmute::<[u64; 4], [u8; 32]>(self.circulating_supply.0) })?;
        // SAFETY: Reinterpreting a [u64; 4] as a [u8; 32] is fine since they have the same size.
        writer.write_all(&unsafe { std::mem::transmute::<[u64; 4], [u8; 32]>(self.maximum_supply.0) })?;
        self.token_scheme.pack(writer)?;
        (self.feature_blocks.len() as u16).pack(writer)?;
        for feature_block in self.feature_blocks.iter() {
            feature_block.pack(writer)?
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let address = Address::unpack_inner::<R, CHECK>(reader)?;
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let native_tokens_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut native_tokens = Vec::with_capacity(native_tokens_len);
        for _ in 0..native_tokens_len {
            native_tokens.push(NativeToken::unpack_inner::<R, CHECK>(reader)?);
        }
        let serial_number = u32::unpack_inner::<R, CHECK>(reader)?;
        let token_tag = <[u8; 12]>::unpack_inner::<R, CHECK>(reader)?;
        let circulating_supply = U256::from_little_endian(&<[u8; 32]>::unpack_inner::<R, CHECK>(reader)?);
        let maximum_supply = U256::from_little_endian(&<[u8; 32]>::unpack_inner::<R, CHECK>(reader)?);
        let token_scheme = TokenScheme::unpack_inner::<R, CHECK>(reader)?;
        let feature_blocks_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut feature_blocks = Vec::with_capacity(feature_blocks_len);
        for _ in 0..feature_blocks_len {
            feature_blocks.push(FeatureBlock::unpack_inner::<R, CHECK>(reader)?);
        }

        Ok(Self {
            address,
            amount,
            native_tokens: native_tokens.into_boxed_slice(),
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
            feature_blocks: feature_blocks.into_boxed_slice(),
        })
    }
}
