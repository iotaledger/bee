// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    http::header::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bee_block::{BlockDto, BlockId};
use lazy_static::lazy_static;
use packable::PackableExt;

use crate::{
    endpoints::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode},
    types::responses::BlockResponse,
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
) -> Result<Response, ApiError> {
    if let Some(value) = headers.get(axum::http::header::ACCEPT) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            return blocks_raw::<B>(block_id, args.clone()).await.map(|r| r.into_response());
        } else {
            blocks_json::<B>(block_id, args.clone())
                .await
                .map(|r| r.into_response())
        }
    } else {
        blocks_json::<B>(block_id, args.clone())
            .await
            .map(|r| r.into_response())
    }
}

pub(crate) async fn blocks_json<B: StorageBackend>(
    block_id: BlockId,
    args: ApiArgsFullNode<B>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&block_id) {
        Some(block) => Ok(Json(BlockResponse(BlockDto::from(&block)))),
        None => Err(ApiError::NotFound),
    }
}

async fn blocks_raw<B: StorageBackend>(
    block_id: BlockId,
    args: ApiArgsFullNode<B>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&block_id) {
        Some(block) => Ok(block.pack_to_vec()),
        None => Err(ApiError::NotFound),
    }
}
