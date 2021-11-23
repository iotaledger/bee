// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{FeatureBlock, FeatureBlocks, NativeToken, NativeTokens},
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
    pub fn finish(self) -> Result<FoundryOutput, Error> {
        Ok(FoundryOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            serial_number: self.serial_number,
            token_tag: self.token_tag,
            circulating_supply: self.circulating_supply,
            maximum_supply: self.maximum_supply,
            token_scheme: self.token_scheme,
            feature_blocks: FeatureBlocks::new(self.feature_blocks)?,
        })
    }
}

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FoundryOutput {
    // Deposit address of the output.
    address: Address,
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    serial_number: u32,
    token_tag: [u8; 12],
    circulating_supply: U256,
    maximum_supply: U256,
    token_scheme: TokenScheme,
    feature_blocks: FeatureBlocks,
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
        // SAFETY: this can't fail as this is a default builder.
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
            + self.native_tokens.packed_len()
            + self.serial_number.packed_len()
            + self.token_tag.packed_len()
            + 32
            + 32
            + self.token_scheme.packed_len()
            + self.feature_blocks.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.address.pack(writer)?;
        self.amount.pack(writer)?;
        self.native_tokens.pack(writer)?;
        self.serial_number.pack(writer)?;
        self.token_tag.pack(writer)?;
        // SAFETY: Reinterpreting a [u64; 4] as a [u8; 32] is fine since they have the same size.
        writer.write_all(&unsafe { std::mem::transmute::<[u64; 4], [u8; 32]>(self.circulating_supply.0) })?;
        // SAFETY: Reinterpreting a [u64; 4] as a [u8; 32] is fine since they have the same size.
        writer.write_all(&unsafe { std::mem::transmute::<[u64; 4], [u8; 32]>(self.maximum_supply.0) })?;
        self.token_scheme.pack(writer)?;
        self.feature_blocks.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let address = Address::unpack_inner::<R, CHECK>(reader)?;
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let native_tokens = NativeTokens::unpack_inner::<R, CHECK>(reader)?;
        let serial_number = u32::unpack_inner::<R, CHECK>(reader)?;
        let token_tag = <[u8; 12]>::unpack_inner::<R, CHECK>(reader)?;
        let circulating_supply = U256::from_little_endian(&<[u8; 32]>::unpack_inner::<R, CHECK>(reader)?);
        let maximum_supply = U256::from_little_endian(&<[u8; 32]>::unpack_inner::<R, CHECK>(reader)?);
        let token_scheme = TokenScheme::unpack_inner::<R, CHECK>(reader)?;
        let feature_blocks = FeatureBlocks::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            address,
            amount,
            native_tokens,
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
            feature_blocks,
        })
    }
}
