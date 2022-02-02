// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{verify_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{
            verify_allowed_unlock_conditions, AddressUnlockCondition, UnlockCondition, UnlockConditionFlags,
            UnlockConditions,
        },
        FoundryId, NativeToken, NativeTokens, OutputAmount,
    },
    Error,
};

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::{Packer, SlicePacker},
    unpacker::Unpacker,
    Packable,
};
use primitive_types::U256;

use alloc::vec::Vec;

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
    amount: OutputAmount,
    native_tokens: Vec<NativeToken>,
    serial_number: u32,
    token_tag: [u8; 12],
    circulating_supply: U256,
    maximum_supply: U256,
    token_scheme: TokenScheme,
    unlock_conditions: Vec<UnlockCondition>,
    feature_blocks: Vec<FeatureBlock>,
}

impl FoundryOutputBuilder {
    ///
    pub fn new(
        amount: u64,
        serial_number: u32,
        token_tag: [u8; 12],
        circulating_supply: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        verify_supply(&circulating_supply, &maximum_supply)?;

        Ok(Self {
            amount: amount.try_into().map_err(Error::InvalidOutputAmount)?,
            native_tokens: Vec::new(),
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
            unlock_conditions: Vec::new(),
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
    pub fn add_unlock_condition(mut self, unlock_condition: UnlockCondition) -> Self {
        self.unlock_conditions.push(unlock_condition);
        self
    }

    ///
    #[inline(always)]
    pub fn with_unlock_conditions(mut self, unlock_conditions: impl IntoIterator<Item = UnlockCondition>) -> Self {
        self.unlock_conditions = unlock_conditions.into_iter().collect();
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
        let unlock_conditions = UnlockConditions::new(self.unlock_conditions)?;

        verify_unlock_conditions(&unlock_conditions)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        verify_allowed_feature_blocks(&feature_blocks, FoundryOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(FoundryOutput {
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            serial_number: self.serial_number,
            token_tag: self.token_tag,
            circulating_supply: self.circulating_supply,
            maximum_supply: self.maximum_supply,
            token_scheme: self.token_scheme,
            unlock_conditions,
            feature_blocks,
        })
    }
}

/// Describes a foundry output that is controlled by an alias.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct FoundryOutput {
    // Amount of IOTA tokens held by the output.
    amount: OutputAmount,
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
    unlock_conditions: UnlockConditions,
    feature_blocks: FeatureBlocks,
}

impl FoundryOutput {
    /// The [`Output`](crate::output::Output) kind of a [`FoundryOutput`].
    pub const KIND: u8 = 5;

    /// The set of allowed [`UnlockCondition`]s for an [`FoundryOutput`].
    const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::ADDRESS;
    /// The set of allowed [`FeatureBlock`]s for an [`FoundryOutput`].
    pub const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::METADATA;

    /// Creates a new [`FoundryOutput`].
    #[inline(always)]
    pub fn new(
        amount: u64,
        serial_number: u32,
        token_tag: [u8; 12],
        circulating_supply: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<Self, Error> {
        FoundryOutputBuilder::new(
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
        amount: u64,
        serial_number: u32,
        token_tag: [u8; 12],
        circulating_supply: U256,
        maximum_supply: U256,
        token_scheme: TokenScheme,
    ) -> Result<FoundryOutputBuilder, Error> {
        FoundryOutputBuilder::new(
            amount,
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
        )
    }

    /// Returns the [`FoundryId`] of the [`FoundryOutput`].
    pub fn id(&self) -> FoundryId {
        let mut bytes = [0u8; FoundryId::LENGTH];
        let mut packer = SlicePacker::new(&mut bytes);

        // SAFETY: packing to an array of the correct length can't fail.
        self.address().pack(&mut packer).unwrap();
        self.serial_number.pack(&mut packer).unwrap();
        self.token_scheme.pack(&mut packer).unwrap();

        FoundryId::new(bytes)
    }

    ///
    #[inline(always)]
    pub fn address(&self) -> &Address {
        // A FoundryOutput must have a AddressUnlockCondition.
        if let UnlockCondition::Address(address) = self.unlock_conditions.get(AddressUnlockCondition::KIND).unwrap() {
            address.address()
        } else {
            unreachable!();
        }
    }

    ///
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
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
    pub fn unlock_conditions(&self) -> &[UnlockCondition] {
        &self.unlock_conditions
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
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.serial_number.pack(packer)?;
        self.token_tag.pack(packer)?;
        self.circulating_supply.pack(packer)?;
        self.maximum_supply.pack(packer)?;
        self.token_scheme.pack(packer)?;
        self.unlock_conditions.pack(packer)?;
        self.feature_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let amount = OutputAmount::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::InvalidOutputAmount(err.into()))?;
        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker)?;
        let serial_number = u32::unpack::<_, VERIFY>(unpacker).infallible()?;
        let token_tag = <[u8; 12]>::unpack::<_, VERIFY>(unpacker).infallible()?;
        let circulating_supply = U256::unpack::<_, VERIFY>(unpacker).infallible()?;
        let maximum_supply = U256::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            verify_supply(&circulating_supply, &maximum_supply).map_err(UnpackError::Packable)?;
        }

        let token_scheme = TokenScheme::unpack::<_, VERIFY>(unpacker)?;

        let unlock_conditions = UnlockConditions::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_unlock_conditions(&unlock_conditions).map_err(UnpackError::Packable)?;
        }

        let feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_feature_blocks(&feature_blocks, FoundryOutput::ALLOWED_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            serial_number,
            token_tag,
            circulating_supply,
            maximum_supply,
            token_scheme,
            unlock_conditions,
            feature_blocks,
        })
    }
}

#[inline]
fn verify_supply(circulating_supply: &U256, maximum_supply: &U256) -> Result<(), Error> {
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

fn verify_unlock_conditions(unlock_conditions: &UnlockConditions) -> Result<(), Error> {
    if let Some(UnlockCondition::Address(unlock_condition)) = unlock_conditions.get(AddressUnlockCondition::KIND) {
        match unlock_condition.address() {
            Address::Alias(_) => {}
            _ => return Err(Error::InvalidAddressKind(unlock_condition.address().kind())),
        };
    } else {
        return Err(Error::MissingAddressUnlockCondition);
    }

    verify_allowed_unlock_conditions(unlock_conditions, FoundryOutput::ALLOWED_UNLOCK_CONDITIONS)
}
