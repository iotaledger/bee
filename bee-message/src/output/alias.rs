// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{
        feature_block::{
            validate_allowed_feature_blocks, FeatureBlock, FeatureBlocks, IssuerFeatureBlock, MetadataFeatureBlock,
            SenderFeatureBlock,
        },
        AliasId, NativeToken, NativeTokens,
    },
    Error,
};

use bee_common::packable::{Packable, Read, Write};

///
const METADATA_LENGTH_MAX: usize = 1024;

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
    pub fn new(amount: u64, alias_id: AliasId, state_controller: Address, governance_controller: Address) -> Self {
        Self {
            amount,
            native_tokens: Vec::new(),
            alias_id,
            state_controller,
            governance_controller,
            state_index: None,
            state_metadata: Vec::new(),
            foundry_counter: None,
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
    pub fn with_state_index(mut self, state_index: u32) -> Self {
        self.state_index.replace(state_index);
        self
    }

    ///
    pub fn with_state_metadata(mut self, state_metadata: Vec<u8>) -> Self {
        self.state_metadata = state_metadata;
        self
    }

    ///
    pub fn with_foundry_counter(mut self, foundry_counter: u32) -> Self {
        self.foundry_counter.replace(foundry_counter);
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
    pub fn finish(self) -> Result<AliasOutput, Error> {
        if self.state_metadata.len() > METADATA_LENGTH_MAX {
            return Err(Error::InvalidMetadataLength(self.state_metadata.len()));
        }

        let feature_blocks = FeatureBlocks::new(self.feature_blocks)?;

        validate_allowed_feature_blocks(&feature_blocks, &AliasOutput::ALLOWED_FEATURE_BLOCKS)?;

        Ok(AliasOutput {
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            alias_id: self.alias_id,
            state_controller: self.state_controller,
            governance_controller: self.governance_controller,
            state_index: self.state_index.unwrap_or(0),
            state_metadata: self.state_metadata.into_boxed_slice(),
            foundry_counter: self.foundry_counter.unwrap_or(0),
            feature_blocks,
        })
    }
}

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
    state_metadata: Box<[u8]>,
    // A counter that denotes the number of foundries created by this alias account.
    foundry_counter: u32,
    //
    feature_blocks: FeatureBlocks,
}

impl AliasOutput {
    /// The [`Output`](crate::output::Output) kind of an [`AliasOutput`].
    pub const KIND: u8 = 4;
    ///
    const ALLOWED_FEATURE_BLOCKS: [u8; 3] = [
        SenderFeatureBlock::KIND,
        IssuerFeatureBlock::KIND,
        MetadataFeatureBlock::KIND,
    ];

    /// Creates a new [`AliasOutput`].
    pub fn new(amount: u64, alias_id: AliasId, state_controller: Address, governance_controller: Address) -> Self {
        // SAFETY: this can't fail as this is a default builder.
        AliasOutputBuilder::new(amount, alias_id, state_controller, governance_controller)
            .finish()
            .unwrap()
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
    pub fn alias_id(&self) -> &AliasId {
        &self.alias_id
    }

    ///
    pub fn state_controller(&self) -> &Address {
        &self.state_controller
    }

    ///
    pub fn governance_controller(&self) -> &Address {
        &self.governance_controller
    }

    ///
    pub fn state_index(&self) -> u32 {
        self.state_index
    }

    ///
    pub fn state_metadata(&self) -> &[u8] {
        &self.state_metadata
    }

    ///
    pub fn foundry_counter(&self) -> u32 {
        self.foundry_counter
    }

    ///
    pub fn feature_blocks(&self) -> &[FeatureBlock] {
        &self.feature_blocks
    }
}

impl Packable for AliasOutput {
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
        let governance_controller = Address::unpack_inner::<R, CHECK>(reader)?;
        let state_index = u32::unpack_inner::<R, CHECK>(reader)?;
        let state_metadata_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;
        if CHECK && state_metadata_len > METADATA_LENGTH_MAX {
            return Err(Error::InvalidMetadataLength(state_metadata_len));
        }
        let mut state_metadata = vec![0u8; state_metadata_len];
        reader.read_exact(&mut state_metadata)?;
        let foundry_counter = u32::unpack_inner::<R, CHECK>(reader)?;
        let feature_blocks = FeatureBlocks::unpack_inner::<R, CHECK>(reader)?;

        if CHECK {
            validate_allowed_feature_blocks(&feature_blocks, &AliasOutput::ALLOWED_FEATURE_BLOCKS)?;
        }

        Ok(Self {
            amount,
            native_tokens,
            alias_id,
            state_controller,
            governance_controller,
            state_index,
            state_metadata: state_metadata.into_boxed_slice(),
            foundry_counter,
            feature_blocks,
        })
    }
}
