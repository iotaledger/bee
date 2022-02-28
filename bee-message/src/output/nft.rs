// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{verify_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::{
            verify_allowed_unlock_conditions, AddressUnlockCondition, UnlockCondition, UnlockConditionFlags,
            UnlockConditions,
        },
        NativeToken, NativeTokens, NftId, OutputAmount,
    },
    Error,
};

use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use alloc::vec::Vec;

///
#[must_use]
pub struct NftOutputBuilder {
    amount: OutputAmount,
    native_tokens: Vec<NativeToken>,
    nft_id: NftId,
    unlock_conditions: Vec<UnlockCondition>,
    feature_blocks: Vec<FeatureBlock>,
    immutable_feature_blocks: Vec<FeatureBlock>,
}

impl NftOutputBuilder {
    ///
    pub fn new(amount: u64, nft_id: NftId) -> Result<NftOutputBuilder, Error> {
        Ok(Self {
            amount: amount.try_into().map_err(Error::InvalidOutputAmount)?,
            native_tokens: Vec::new(),
            nft_id,
            unlock_conditions: Vec::new(),
            feature_blocks: Vec::new(),
            immutable_feature_blocks: Vec::new(),
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
    #[inline(always)]
    pub fn add_immutable_feature_block(mut self, immutable_feature_block: FeatureBlock) -> Self {
        self.immutable_feature_blocks.push(immutable_feature_block);
        self
    }

    ///
    #[inline(always)]
    pub fn with_immutable_feature_blocks(
        mut self,
        immutable_feature_blocks: impl IntoIterator<Item = FeatureBlock>,
    ) -> Self {
        self.immutable_feature_blocks = immutable_feature_blocks.into_iter().collect();
        self
    }

    ///
    pub fn finish(self) -> Result<NftOutput, Error> {
        let unlock_conditions = UnlockConditions::new(self.unlock_conditions)?;

        verify_unlock_conditions(&unlock_conditions, &self.nft_id)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        verify_allowed_feature_blocks(&feature_blocks, NftOutput::ALLOWED_FEATURE_BLOCKS)?;

        let immutable_feature_blocks = FeatureBlocks::new(self.immutable_feature_blocks)?;

        verify_allowed_feature_blocks(&immutable_feature_blocks, NftOutput::ALLOWED_IMMUTABLE_FEATURE_BLOCKS)?;

        Ok(NftOutput {
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            nft_id: self.nft_id,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        })
    }
}

/// Describes an NFT output, a globally unique token with metadata attached.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct NftOutput {
    // Amount of IOTA tokens held by the output.
    amount: OutputAmount,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // Unique identifier of the NFT.
    nft_id: NftId,
    unlock_conditions: UnlockConditions,
    feature_blocks: FeatureBlocks,
    immutable_feature_blocks: FeatureBlocks,
}

impl NftOutput {
    /// The [`Output`](crate::output::Output) kind of an [`NftOutput`].
    pub const KIND: u8 = 6;

    /// The set of allowed [`UnlockCondition`]s for an [`NftOutput`].
    const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::ADDRESS
        .union(UnlockConditionFlags::STORAGE_DEPOSIT_RETURN)
        .union(UnlockConditionFlags::TIMELOCK)
        .union(UnlockConditionFlags::EXPIRATION);
    /// The set of allowed [`FeatureBlock`]s for an [`NftOutput`].
    pub const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER
        .union(FeatureBlockFlags::METADATA)
        .union(FeatureBlockFlags::TAG);
    /// The set of allowed immutable [`FeatureBlock`]s for an [`NftOutput`].
    pub const ALLOWED_IMMUTABLE_FEATURE_BLOCKS: FeatureBlockFlags =
        FeatureBlockFlags::ISSUER.union(FeatureBlockFlags::METADATA);

    /// Creates a new [`NftOutput`].
    #[inline(always)]
    pub fn new(amount: u64, nft_id: NftId) -> Result<Self, Error> {
        NftOutputBuilder::new(amount, nft_id)?.finish()
    }

    /// Creates a new [`NftOutputBuilder`].
    #[inline(always)]
    pub fn build(amount: u64, nft_id: NftId) -> Result<NftOutputBuilder, Error> {
        NftOutputBuilder::new(amount, nft_id)
    }

    ///
    #[inline(always)]
    pub fn amount(&self) -> u64 {
        self.amount.get()
    }

    ///
    #[inline(always)]
    pub fn native_tokens(&self) -> &NativeTokens {
        &self.native_tokens
    }

    ///
    #[inline(always)]
    pub fn nft_id(&self) -> &NftId {
        &self.nft_id
    }

    // TODO bring back as part of #1117
    // ///
    // #[inline(always)]
    // pub fn immutable_metadata(&self) -> &[u8] {
    //     &self.immutable_metadata
    // }

    ///
    #[inline(always)]
    pub fn unlock_conditions(&self) -> &UnlockConditions {
        &self.unlock_conditions
    }

    ///
    #[inline(always)]
    pub fn feature_blocks(&self) -> &FeatureBlocks {
        &self.feature_blocks
    }

    ///
    #[inline(always)]
    pub fn immutable_feature_blocks(&self) -> &FeatureBlocks {
        &self.immutable_feature_blocks
    }

    ///
    #[inline(always)]
    pub fn address(&self) -> &Address {
        // An NftOutput must have an AddressUnlockCondition.
        if let UnlockCondition::Address(address) = self.unlock_conditions.get(AddressUnlockCondition::KIND).unwrap() {
            address.address()
        } else {
            unreachable!();
        }
    }
}

impl Packable for NftOutput {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.nft_id.pack(packer)?;
        self.unlock_conditions.pack(packer)?;
        self.feature_blocks.pack(packer)?;
        self.immutable_feature_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let amount = OutputAmount::unpack::<_, VERIFY>(unpacker).map_packable_err(Error::InvalidOutputAmount)?;
        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker)?;
        let nft_id = NftId::unpack::<_, VERIFY>(unpacker).infallible()?;
        let unlock_conditions = UnlockConditions::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_unlock_conditions(&unlock_conditions, &nft_id).map_err(UnpackError::Packable)?;
        }

        let feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_feature_blocks(&feature_blocks, NftOutput::ALLOWED_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        let immutable_feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_allowed_feature_blocks(&immutable_feature_blocks, NftOutput::ALLOWED_IMMUTABLE_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            nft_id,
            unlock_conditions,
            feature_blocks,
            immutable_feature_blocks,
        })
    }
}

fn verify_unlock_conditions(unlock_conditions: &UnlockConditions, nft_id: &NftId) -> Result<(), Error> {
    if let Some(UnlockCondition::Address(unlock_condition)) = unlock_conditions.get(AddressUnlockCondition::KIND) {
        if let Address::Nft(nft_address) = unlock_condition.address() {
            if nft_address.nft_id() == nft_id {
                return Err(Error::SelfDepositNft(*nft_id));
            }
        }
    } else {
        return Err(Error::MissingAddressUnlockCondition);
    }

    verify_allowed_unlock_conditions(unlock_conditions, NftOutput::ALLOWED_UNLOCK_CONDITIONS)
}
