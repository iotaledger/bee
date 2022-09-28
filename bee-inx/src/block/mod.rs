// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
    // TODO
    pub async fn listen_to_blocks(&mut self) -> Result<impl Stream<Item = Result<Block, Error>>, Error> {
        Ok(self
            .client
            .listen_to_blocks(inx::NoParams {})
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }

    // TODO
    pub async fn listen_to_solid_blocks(&mut self) -> Result<impl Stream<Item = Result<BlockMetadata, Error>>, Error> {
        Ok(self
            .client
            .listen_to_solid_blocks(inx::NoParams {})
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }

    // TODO
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

    // TODO
    pub async fn read_block(&mut self, block_id: bee::BlockId) -> Result<Raw<bee::Block>, Error> {
        Ok(self
            .client
            .read_block(inx::BlockId::from(block_id))
            .await?
            .into_inner()
            .data
            .into())
    }

    // TODO
    pub async fn read_block_metadata(&mut self, block_id: bee::BlockId) -> Result<BlockMetadata, Error> {
        Ok(self
            .client
            .read_block_metadata(inx::BlockId::from(block_id))
            .await?
            .into_inner()
            .try_into()?)
    }

    // TODO
    pub async fn submit_block(&mut self, raw_block: Raw<bee::Block>) -> Result<bee::BlockId, Error> {
        Ok(self
            .client
            .submit_block(inx::RawBlock { data: raw_block.data() })
            .await?
            .into_inner()
            .try_into()?)
    }
}
