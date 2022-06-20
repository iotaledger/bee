// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_block::BlockId;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode, CONFIRMED_THRESHOLD},
    types::responses::TipsResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/tips", get(tips::<B>))
}

async fn tips<B: StorageBackend>(Extension(args): Extension<ApiArgsFullNode<B>>) -> Result<TipsResponse, ApiError> {
    if !args.tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(ApiError::ServiceUnavailable("the node is not synchronized"));
    }
    match args.tangle.get_blocks_to_approve().await {
        Some(tips) => Ok(TipsResponse {
            tips: tips.iter().map(BlockId::to_string).collect(),
        }),
        None => Err(ApiError::ServiceUnavailable("no tips available")),
    }
}
