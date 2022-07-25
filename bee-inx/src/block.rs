// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

/// The [`Block`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The [`BlockId`](bee::BlockId) of the block.
    pub block_id: bee::BlockId,
    /// The complete [`Block`](bee::Block).
    pub block: bee::Block,
    /// The raw bytes of the block.
    pub raw: Vec<u8>,
}

/// The [`BlockWithMetadata`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockWithMetadata {
    /// The [`Metadata`](crate::BlockMetadata) of the block.
    pub metadata: crate::BlockMetadata,
    /// The complete [`Block`](bee::Block).
    pub block: bee::Block,
    /// The raw bytes of the block.
    pub raw: Vec<u8>,
}

impl TryFrom<proto::BlockWithMetadata> for BlockWithMetadata {
    type Error = bee::InxError;

    fn try_from(value: proto::BlockWithMetadata) -> Result<Self, Self::Error> {
        let raw = value.block.ok_or(bee::InxError::MissingField("block"))?;
        Ok(BlockWithMetadata {
            metadata: value
                .metadata
                .ok_or(Self::Error::MissingField("metadata"))?
                .try_into()?,
            block: raw.clone().try_into()?,
            raw: raw.data,
        })
    }
}

impl TryFrom<proto::Block> for Block {
    type Error = bee::InxError;

    fn try_from(value: proto::Block) -> Result<Self, Self::Error> {
        let raw = value.block.ok_or(Self::Error::MissingField("block"))?;
        Ok(Block {
            block_id: value
                .block_id
                .ok_or(Self::Error::MissingField("block_id"))?
                .try_into()?,
            block: raw.clone().try_into()?,
            raw: raw.data,
        })
    }
}
