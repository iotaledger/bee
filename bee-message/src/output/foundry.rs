// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{validate_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        NativeToken, NativeTokens,
    },
    Error,
};

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};
use primitive_types::U256;

///
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidTokenSchemeKind)]
pub enum TokenScheme {
    ///
    Simple = 0,
}

///
#[must_use]
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
    ) -> Result<FoundryOutputBuilder, Error> {
        validate_address(&address)?;
        validate_supply(&circulating_supply, &maximum_supply)?;

        Ok(Self {
            address,
            amount,
            native_tokens: Vec::new(),
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
            feature_blocks: Vec::new(),
        })
    }

    ///
    #[inline(always)]
    pub fn add_native_token(mut self, native_token: NativeToken) -> Self {
        self.native_tokens.push(native_token);
        self
    }

    ///
    #[inline(always)]
    pub fn with_native_tokens(mut self, native_tokens: impl IntoIterator<Item = NativeToken>) -> Self {
        self.native_tokens = native_tokens.into_iter().collect();
        self
    }

    ///
    #[inline(always)]
    pub fn add_feature_block(mut self, feature_block: FeatureBlock) -> Self {
        self.feature_blocks.push(feature_block);
        self
    }

    ///
    #[inline(always)]
    pub fn with_feature_blocks(mut self, feature_blocks: impl IntoIterator<Item = FeatureBlock>) -> Self {
        self.feature_blocks = feature_blocks.into_iter().collect();
        self
    }

    ///
    pub fn finish(self) -> Result<FoundryOutput, Error> {
        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        validate_allowed_feature_blocks(&feature_blocks, FoundryOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(FoundryOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            serial_number: self.serial_number,
            token_tag: self.token_tag,
            circulating_supply: self.circulating_supply,
            maximum_supply: self.maximum_supply,
            token_scheme: self.token_scheme,
            feature_blocks,
        })
    }
}

/// Describes a foundry output that is controlled by an alias.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FoundryOutput {
    // Deposit address of the output.
    address: Address,
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // The serial number of the foundry with respect to the controlling alias.
    serial_number: u32,
    // Data that is always the last 12 bytes of ID of the tokens produced by this foundry.
    token_tag: [u8; 12],
    // Circulating supply of tokens controlled by this foundry.
    circulating_supply: U256,
    // Maximum supply of tokens controlled by this foundry.
    maximum_supply: U256,
    token_scheme: TokenScheme,
    feature_blocks: FeatureBlocks,
}

impl FoundryOutput {
    /// The [`Output`](crate::output::Output) kind of a [`FoundryOutput`].
    pub const KIND: u8 = 5;

    /// The set of allowed [`FeatureBlock`]s for an [`FoundryOutput`].
    pub const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::METADATA;

    /// Creates a new [`FoundryOutput`].
    #[inline(always)]
    pub fn new(
        address: Address,
        amount: u64,
        serial_number: u32,
        token_tag: [u8; 12],
        circulating_supply: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<Self, Error> {
        FoundryOutputBuilder::new(
            address,
            amount,
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
        )?
        .finish()
    }

    /// Creates a new [`FoundryOutputBuilder`].
    #[inline(always)]
    pub fn build(
        address: Address,
        amount: u64,
        serial_number: u32,
        token_tag: [u8; 12],
        circulating_supply: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        FoundryOutputBuilder::new(
            address,
            amount,
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
        )
    }

    ///
    #[inline(always)]
    pub fn address(&self) -> &Address {
        &self.address
    }

    ///
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount
    }

    ///
    #[inline(always)]
    pub fn native_tokens(&self) -> &[NativeToken] {
        &self.native_tokens
    }

    ///
    #[inline(always)]
    pub fn serial_number(&self) -> u32 {
        self.serial_number
    }

    ///
    #[inline(always)]
    pub fn token_tag(&self) -> &[u8; 12] {
        &self.token_tag
    }

    ///
    #[inline(always)]
    pub fn circulating_supply(&self) -> &U256 {
        &self.circulating_supply
    }

    ///
    #[inline(always)]
    pub fn maximum_supply(&self) -> &U256 {
        &self.maximum_supply
    }

    ///
    #[inline(always)]
    pub fn token_scheme(&self) -> TokenScheme {
        self.token_scheme
    }

    ///
    #[inline(always)]
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl Packable for FoundryOutput {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.address.pack(packer)?;
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.serial_number.pack(packer)?;
        self.token_tag.pack(packer)?;
        self.circulating_supply.pack(packer)?;
        self.maximum_supply.pack(packer)?;
        self.token_scheme.pack(packer)?;
        self.feature_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let address = Address::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            validate_address(&address).map_err(UnpackError::Packable)?;
        }

        let amount = u64::unpack::<_, VERIFY>(unpacker).infallible()?;
        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker)?;
        let serial_number = u32::unpack::<_, VERIFY>(unpacker).infallible()?;
        let token_tag = <[u8; 12]>::unpack::<_, VERIFY>(unpacker).infallible()?;
        let circulating_supply = U256::unpack::<_, VERIFY>(unpacker).infallible()?;
        let maximum_supply = U256::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            validate_supply(&circulating_supply, &maximum_supply).map_err(UnpackError::Packable)?;
        }

        let token_scheme = TokenScheme::unpack::<_, VERIFY>(unpacker)?;
        let feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            validate_allowed_feature_blocks(&feature_blocks, FoundryOutput::ALLOWED_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

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

#[inline]
fn validate_address(address: &Address) -> Result<(), Error> {
    match address {
        Address::Alias(_) => {}
        _ => return Err(Error::InvalidAddressKind(address.kind())),
    };

    Ok(())
}

#[inline]
fn validate_supply(circulating_supply: &U256, maximum_supply: &U256) -> Result<(), Error> {
    if maximum_supply.is_zero() {
        return Err(Error::InvalidFoundryOutputSupply {
            circulating: *circulating_supply,
            max: *maximum_supply,
        });
    }

    if circulating_supply > maximum_supply {
        return Err(Error::InvalidFoundryOutputSupply {
            circulating: *circulating_supply,
            max: *maximum_supply,
        });
    }

    Ok(())
}
