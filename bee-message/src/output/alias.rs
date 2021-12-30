// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{validate_allowed_feature_blocks, FeatureBlock, FeatureBlockFlags, FeatureBlocks},
        AliasId, NativeToken, NativeTokens,
    },
    Error,
};

use bee_common::packable::{Read, Write};
use bee_packable::{
    bounded::BoundedU32,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
};

///
pub struct AliasOutputBuilder {
    amount: u64,
    native_tokens: Vec<NativeToken>,
    alias_id: AliasId,
    state_controller: Address,
    governance_controller: Address,
    state_index: Option<u32>,
    state_metadata: Vec<u8>,
    foundry_counter: Option<u32>,
    feature_blocks: Vec<FeatureBlock>,
}

impl AliasOutputBuilder {
    ///
    pub fn new(
        amount: u64,
        alias_id: AliasId,
        state_controller: Address,
        governance_controller: Address,
    ) -> Result<AliasOutputBuilder, Error> {
        validate_controller(&state_controller, &alias_id)?;
        validate_controller(&governance_controller, &alias_id)?;

        Ok(Self {
            amount,
            native_tokens: Vec::new(),
            alias_id,
            state_controller,
            governance_controller,
            state_index: None,
            state_metadata: Vec::new(),
            foundry_counter: None,
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
    pub fn with_state_index(mut self, state_index: u32) -> Self {
        self.state_index.replace(state_index);
        self
    }

    ///
    #[inline(always)]
    pub fn with_state_metadata(mut self, state_metadata: Vec<u8>) -> Self {
        self.state_metadata = state_metadata;
        self
    }

    ///
    #[inline(always)]
    pub fn with_foundry_counter(mut self, foundry_counter: u32) -> Self {
        self.foundry_counter.replace(foundry_counter);
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
    pub fn finish(self) -> Result<AliasOutput, Error> {
        let state_index = self.state_index.unwrap_or(0);
        let foundry_counter = self.foundry_counter.unwrap_or(0);

        let state_metadata = self
            .state_metadata
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidStateMetadataLength)?;

        validate_index_counter(&self.alias_id, state_index, foundry_counter)?;

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        validate_allowed_feature_blocks(&feature_blocks, AliasOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(AliasOutput {
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            alias_id: self.alias_id,
            state_controller: self.state_controller,
            governance_controller: self.governance_controller,
            state_index,
            state_metadata,
            foundry_counter,
            feature_blocks,
        })
    }
}

pub(crate) type StateMetadataLength = BoundedU32<0, { AliasOutput::STATE_METADATA_LENGTH_MAX }>;

/// Describes an alias account in the ledger that can be controlled by the state and governance controllers.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct AliasOutput {
    // Amount of IOTA tokens held by the output.
    amount: u64,
    // Native tokens held by the output.
    native_tokens: NativeTokens,
    // Unique identifier of the alias.
    alias_id: AliasId,
    //
    state_controller: Address,
    //
    governance_controller: Address,
    // A counter that must increase by 1 every time the alias is state transitioned.
    state_index: u32,
    // Metadata that can only be changed by the state controller.
    state_metadata: BoxedSlicePrefix<u8, StateMetadataLength>,
    // A counter that denotes the number of foundries created by this alias account.
    foundry_counter: u32,
    //
    feature_blocks: FeatureBlocks,
}

impl AliasOutput {
    /// The [`Output`](crate::output::Output) kind of an [`AliasOutput`].
    pub const KIND: u8 = 4;
    /// Maximum possible length in bytes of the state metadata.
    pub const STATE_METADATA_LENGTH_MAX: u32 = 1024;

    /// The set of allowed [`FeatureBlock`]s for an [`AliasOutput`].
    const ALLOWED_FEATURE_BLOCKS: FeatureBlockFlags = FeatureBlockFlags::SENDER
        .union(FeatureBlockFlags::ISSUER)
        .union(FeatureBlockFlags::METADATA);

    /// Creates a new [`AliasOutput`].
    #[inline(always)]
    pub fn new(
        amount: u64,
        alias_id: AliasId,
        state_controller: Address,
        governance_controller: Address,
    ) -> Result<Self, Error> {
        AliasOutputBuilder::new(amount, alias_id, state_controller, governance_controller)?.finish()
    }

    /// Creates a new [`AliasOutputBuilder`].
    #[inline(always)]
    pub fn build(
        amount: u64,
        alias_id: AliasId,
        state_controller: Address,
        governance_controller: Address,
    ) -> Result<AliasOutputBuilder, Error> {
        AliasOutputBuilder::new(amount, alias_id, state_controller, governance_controller)
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
    pub fn alias_id(&self) -> &AliasId {
        &self.alias_id
    }

    ///
    #[inline(always)]
    pub fn state_controller(&self) -> &Address {
        &self.state_controller
    }

    ///
    #[inline(always)]
    pub fn governance_controller(&self) -> &Address {
        &self.governance_controller
    }

    ///
    #[inline(always)]
    pub fn state_index(&self) -> u32 {
        self.state_index
    }

    ///
    #[inline(always)]
    pub fn state_metadata(&self) -> &[u8] {
        &self.state_metadata
    }

    ///
    #[inline(always)]
    pub fn foundry_counter(&self) -> u32 {
        self.foundry_counter
    }

    ///
    #[inline(always)]
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl bee_packable::Packable for AliasOutput {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.amount.pack(packer)?;
        self.native_tokens.pack(packer)?;
        self.alias_id.pack(packer)?;
        self.state_controller.pack(packer)?;
        self.governance_controller.pack(packer)?;
        self.state_index.pack(packer)?;
        self.state_metadata.pack(packer)?;
        self.foundry_counter.pack(packer)?;
        self.feature_blocks.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let amount = u64::unpack::<_, VERIFY>(unpacker).infallible()?;
        let native_tokens = NativeTokens::unpack::<_, VERIFY>(unpacker)?;
        let alias_id = AliasId::unpack::<_, VERIFY>(unpacker).infallible()?;
        let state_controller = Address::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            validate_controller(&state_controller, &alias_id).map_err(UnpackError::Packable)?;
        }

        let governance_controller = Address::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            validate_controller(&governance_controller, &alias_id).map_err(UnpackError::Packable)?;
        }

        let state_index = u32::unpack::<_, VERIFY>(unpacker).infallible()?;
        let state_metadata = BoxedSlicePrefix::<u8, StateMetadataLength>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::InvalidStateMetadataLength(err.into_prefix().into()))?;

        let foundry_counter = u32::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            validate_index_counter(&alias_id, state_index, foundry_counter).map_err(UnpackError::Packable)?;
        }

        let feature_blocks = FeatureBlocks::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            validate_allowed_feature_blocks(&feature_blocks, AliasOutput::ALLOWED_FEATURE_BLOCKS)
                .map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            alias_id,
            state_controller,
            governance_controller,
            state_index,
            state_metadata,
            foundry_counter,
            feature_blocks,
        })
    }
}

impl bee_common::packable::Packable for AliasOutput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.amount.packed_len()
            + self.native_tokens.packed_len()
            + self.alias_id.packed_len()
            + self.state_controller.packed_len()
            + self.governance_controller.packed_len()
            + self.state_index.packed_len()
            + 0u32.packed_len()
            + self.state_metadata.len()
            + self.foundry_counter.packed_len()
            + self.feature_blocks.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.amount.pack(writer)?;
        self.native_tokens.pack(writer)?;
        self.alias_id.pack(writer)?;
        self.state_controller.pack(writer)?;
        self.governance_controller.pack(writer)?;
        self.state_index.pack(writer)?;
        0u32.pack(writer)?;
        writer.write_all(&self.state_metadata)?;
        self.foundry_counter.pack(writer)?;
        self.feature_blocks.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let amount = u64::unpack_inner::<R, CHECK>(reader)?;
        let native_tokens = NativeTokens::unpack_inner::<R, CHECK>(reader)?;
        let alias_id = AliasId::unpack_inner::<R, CHECK>(reader)?;
        let state_controller = Address::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_controller(&state_controller, &alias_id)?;
        }

        let governance_controller = Address::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_controller(&governance_controller, &alias_id)?;
        }

        let state_index = u32::unpack_inner::<R, CHECK>(reader)?;
        let state_metadata_length = u32::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK {
            validate_state_metadata_length(state_metadata_length)?;
        }

        let mut state_metadata = vec![0u8; state_metadata_length];
        reader.read_exact(&mut state_metadata)?;
        let foundry_counter = u32::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_index_counter(&alias_id, state_index, foundry_counter)?;
        }

        let feature_blocks = FeatureBlocks::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_allowed_feature_blocks(&feature_blocks, AliasOutput::ALLOWED_FEATURE_BLOCKS)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            alias_id,
            state_controller,
            governance_controller,
            state_index,
            state_metadata: state_metadata
                .into_boxed_slice()
                .try_into()
                .map_err(Error::InvalidStateMetadataLength)?,
            foundry_counter,
            feature_blocks,
        })
    }
}

#[inline]
fn validate_state_metadata_length(state_metadata_length: usize) -> Result<(), Error> {
    StateMetadataLength::try_from(state_metadata_length).map_err(Error::InvalidStateMetadataLength)?;

    Ok(())
}

#[inline]
fn validate_index_counter(alias_id: &AliasId, state_index: u32, foundry_counter: u32) -> Result<(), Error> {
    if alias_id.as_ref().iter().all(|&b| b == 0) && (state_index != 0 || foundry_counter != 0) {
        return Err(Error::NonZeroStateIndexOrFoundryCounter);
    }

    Ok(())
}

#[inline]
fn validate_controller(controller: &Address, alias_id: &AliasId) -> Result<(), Error> {
    match controller {
        Address::Ed25519(_) => {}
        Address::Alias(address) => {
            if address.id() == alias_id {
                return Err(Error::SelfControlledAliasOutput(*alias_id));
            }
        }
        _ => return Err(Error::InvalidControllerKind(controller.kind())),
    };

    Ok(())
}
