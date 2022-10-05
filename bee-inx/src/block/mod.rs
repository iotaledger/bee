// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// A module that provides block related INX responses.
pub mod responses;

use futures::stream::{Stream, StreamExt};

pub use self::responses::*;
use crate::{
    bee,
    client::{try_from_inx_type, Inx},
    error::Error,
    inx,
    raw::Raw,
};

impl Inx {
    /// Listens to all blocks.
    pub async fn listen_to_blocks(&mut self) -> Result<impl Stream<Item = Result<Block, Error>>, Error> {
        Ok(self
            .client
            .listen_to_blocks(inx::NoParams {})
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }

    /// Listens to solid blocks.
    pub async fn listen_to_solid_blocks(&mut self) -> Result<impl Stream<Item = Result<BlockMetadata, Error>>, Error> {
        Ok(self
            .client
            .listen_to_solid_blocks(inx::NoParams {})
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }

    /// Listens to referenced blocks.
    pub async fn listen_to_referenced_blocks(
        &mut self,
    ) -> Result<impl Stream<Item = Result<BlockMetadata, Error>>, Error> {
        Ok(self
            .client
            .listen_to_referenced_blocks(inx::NoParams {})
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }

    /// Requests the block with the given block id.
    pub async fn read_block(&mut self, block_id: bee::BlockId) -> Result<Raw<bee::Block>, Error> {
        Ok(self
            .client
            .read_block(inx::BlockId::from(block_id))
            .await?
            .into_inner()
            .data
            .into())
    }

    /// Requests the metadata of the block with the given block id.
    pub async fn read_block_metadata(&mut self, block_id: bee::BlockId) -> Result<BlockMetadata, Error> {
        Ok(self
            .client
            .read_block_metadata(inx::BlockId::from(block_id))
            .await?
            .into_inner()
            .try_into()?)
    }

    /// Submits a block and returns its corresponding block id.
    pub async fn submit_block(&mut self, raw_block: Raw<bee::Block>) -> Result<bee::BlockId, Error> {
        Ok(self
            .client
            .submit_block(inx::RawBlock { data: raw_block.data() })
            .await?
            .into_inner()
            .try_into()?)
    }
}
