// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::Address,
    output::{AliasId, FeatureBlock, NativeToken, NativeTokens},
    Error,
};

use bee_common::packable::{Packable, Read, Write};

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
        Ok(AliasOutput {
            amount: self.amount,
            native_tokens: NativeTokens::new(self.native_tokens)?,
            alias_id: self.alias_id,
            state_controller: self.state_controller,
            governance_controller: self.governance_controller,
            state_index: self.state_index.unwrap_or(0),
            state_metadata: self.state_metadata.into_boxed_slice(),
            foundry_counter: self.foundry_counter.unwrap_or(0),
            feature_blocks: self.feature_blocks.into_boxed_slice(),
        })
    }
}

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct AliasOutput {
    amount: u64,
    native_tokens: NativeTokens,
    alias_id: AliasId,
    state_controller: Address,
    governance_controller: Address,
    state_index: u32,
    state_metadata: Box<[u8]>,
    foundry_counter: u32,
    feature_blocks: Box<[FeatureBlock]>,
}

impl AliasOutput {
    /// The output kind of an `AliasOutput`.
    pub const KIND: u8 = 3;

    /// Creates a new `AliasOutput`.
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
            + 0u16.packed_len()
            + self.feature_blocks.iter().map(Packable::packed_len).sum::<usize>()
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
        (self.feature_blocks.len() as u16).pack(writer)?;
        for feature_block in self.feature_blocks.iter() {
            feature_block.pack(writer)?
        }

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
        let mut state_metadata = vec![0u8; state_metadata_len];
        reader.read_exact(&mut state_metadata)?;
        let foundry_counter = u32::unpack_inner::<R, CHECK>(reader)?;
        let feature_blocks_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut feature_blocks = Vec::with_capacity(feature_blocks_len);
        for _ in 0..feature_blocks_len {
            feature_blocks.push(FeatureBlock::unpack_inner::<R, CHECK>(reader)?);
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
            feature_blocks: feature_blocks.into_boxed_slice(),
        })
    }
}
