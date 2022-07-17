// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::Extension,
    http::header::{HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use bee_block::{BlockDto, BlockId};
use lazy_static::lazy_static;
use packable::PackableExt;

use crate::{
    error::ApiError, extractors::path::CustomPath, storage::StorageBackend, types::responses::BlockResponse,
    ApiArgsFullNode,
};

lazy_static! {
    pub(crate) static ref BYTE_CONTENT_HEADER: HeaderValue =
        HeaderValue::from_str("application/vnd.iota.serializer-v1").unwrap();
}

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/blocks/:block_id", get(blocks::<B>))
}

async fn blocks<B: StorageBackend>(
    headers: HeaderMap,
    CustomPath(block_id): CustomPath<BlockId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<BlockResponse, ApiError> {
    if let Some(value) = headers.get(axum::http::header::ACCEPT) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            return blocks_raw::<B>(block_id, args.clone()).await;
        }
    }

    blocks_json::<B>(block_id, args.clone()).await
}

pub(crate) async fn blocks_json<B: StorageBackend>(
    block_id: BlockId,
    args: ApiArgsFullNode<B>,
) -> Result<BlockResponse, ApiError> {
    match args.tangle.get(&block_id) {
        Some(block) => Ok(BlockResponse::Json(BlockDto::from(&block))),
        None => Err(ApiError::NotFound),
    }
}

pub(crate) async fn blocks_raw<B: StorageBackend>(
    block_id: BlockId,
    args: ApiArgsFullNode<B>,
) -> Result<BlockResponse, ApiError> {
    match args.tangle.get(&block_id) {
        Some(block) => Ok(BlockResponse::Raw(block.pack_to_vec())),
        None => Err(ApiError::NotFound),
    }
}
