// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{validate_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        unlock_condition::UnlockConditionFlags,
        NativeToken, NativeTokens, NftId,
    },
    Error,
};

use packable::{
    bounded::BoundedU32,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
    Packable,
};

///
#[must_use]
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
    pub fn finish(self) -> Result<NftOutput, Error> {
        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        validate_allowed_feature_blocks(&feature_blocks, NftOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(NftOutput {
            address: self.address,
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            nft_id: self.nft_id,
            immutable_metadata: self
                .immutable_metadata
                .into_boxed_slice()
                .try_into()
                .map_err(Error::InvalidImmutableMetadataLength)?,
            feature_blocks,
        })
    }
}

pub(crate) type ImmutableMetadataLength = BoundedU32<0, { NftOutput::IMMUTABLE_METADATA_LENGTH_MAX }>;

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
    immutable_metadata: BoxedSlicePrefix<u8, ImmutableMetadataLength>,
    feature_blocks: FeatureBlocks,
}

impl NftOutput {
    /// The [`Output`](crate::output::Output) kind of a [`NftOutput`].
    pub const KIND: u8 = 6;
    ///
    pub const IMMUTABLE_METADATA_LENGTH_MAX: u32 = 1024;

    /// The set of allowed [`UnlockCondition`]s for an [`NftOutput`].
    const ALLOWED_UNLOCK_CONDITIONS: UnlockConditionFlags = UnlockConditionFlags::DUST_DEPOSIT_RETURN
        .union(UnlockConditionFlags::TIMELOCK)
        .union(UnlockConditionFlags::EXPIRATION);
    /// The set of allowed [`FeatureBlock`]s for an [`NftOutput`].
    const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER
        .union(FeatureBlockFlags::ISSUER)
        .union(FeatureBlockFlags::METADATA)
        .union(FeatureBlockFlags::TAG);

    /// Creates a new [`NftOutput`].
    #[inline(always)]
    pub fn new(address: Address, amount: u64, nft_id: NftId, immutable_metadata: Vec<u8>) -> Result<Self, Error> {
        NftOutputBuilder::new(address, amount, nft_id, immutable_metadata)?.finish()
    }

    /// Creates a new [`NftOutputBuilder`].
    #[inline(always)]
    pub fn build(
        address: Address,
        amount: u64,
        nft_id: NftId,
        immutable_metadata: Vec<u8>,
    ) -> Result<NftOutputBuilder, Error> {
        NftOutputBuilder::new(address, amount, nft_id, immutable_metadata)
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
    pub fn nft_id(&self) -> &NftId {
        &self.nft_id
    }

    ///
    #[inline(always)]
    pub fn immutable_metadata(&self) -> &[u8] {
        &self.immutable_metadata
    }

    ///
    #[inline(always)]
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl Packable for NftOutput {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.address.pack(packer)?;
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.nft_id.pack(packer)?;
        self.immutable_metadata.pack(packer)?;
        self.feature_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let address = Address::unpack::<_, VERIFY>(unpacker)?;
        let amount = u64::unpack::<_, VERIFY>(unpacker).infallible()?;
        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker)?;
        let nft_id = NftId::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            validate_address(&address, &nft_id).map_err(UnpackError::Packable)?;
        }

        let immutable_metadata = BoxedSlicePrefix::<u8, ImmutableMetadataLength>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::InvalidImmutableMetadataLength(err.into_prefix().into()))?;

        let feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            validate_allowed_feature_blocks(&feature_blocks, NftOutput::ALLOWED_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            address,
            amount,
            native_tokens,
            nft_id,
            immutable_metadata,
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
    ImmutableMetadataLength::try_from(immutable_metadata_length).map_err(Error::InvalidImmutableMetadataLength)?;

    Ok(())
}
