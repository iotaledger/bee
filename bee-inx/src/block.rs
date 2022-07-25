// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::{field, Raw};

/// The [`Block`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The [`BlockId`](bee::BlockId) of the block.
    pub block_id: bee::BlockId,
    /// The complete [`Block`](bee::Block) as raw bytes.
    pub block: Raw<bee::Block>,
}

/// The [`BlockWithMetadata`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockWithMetadata {
    /// The [`Metadata`](crate::BlockMetadata) of the block.
    pub metadata: crate::BlockMetadata,
    /// The complete [`Block`](bee::Block) as raw bytes.
    pub block: Raw<bee::Block>,
}

impl TryFrom<proto::BlockWithMetadata> for BlockWithMetadata {
    type Error = bee::InxError;

    fn try_from(value: proto::BlockWithMetadata) -> Result<Self, Self::Error> {
        Ok(BlockWithMetadata {
            metadata: field!(value.metadata).try_into()?,
            block: field!(value.block).data.into(),
        })
    }
}

impl TryFrom<proto::Block> for Block {
    type Error = bee::InxError;

    fn try_from(value: proto::Block) -> Result<Self, Self::Error> {
        Ok(Block {
            block_id: field!(value.block_id).try_into()?,
            block: field!(value.block).data.into(),
        })
    }
}
