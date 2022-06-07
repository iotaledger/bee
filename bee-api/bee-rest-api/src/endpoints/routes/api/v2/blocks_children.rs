// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_block::BlockId;

use crate::{
    endpoints::{
        error::ApiError, extractors::path::CustomPath, routes::api::v2::MAX_RESPONSE_RESULTS, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::responses::BlockChildrenResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/blocks/:block_id/children", get(blocks_children::<B>))
}

async fn blocks_children<B: StorageBackend>(
    CustomPath(block_id): CustomPath<BlockId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let all_children = Vec::from_iter(args.tangle.get_children(&block_id).unwrap_or_default());

    let truncated_children = all_children
        .iter()
        .take(MAX_RESPONSE_RESULTS)
        .map(BlockId::to_string)
        .collect::<Vec<String>>();

    Ok(Json(BlockChildrenResponse {
        block_id: block_id.to_string(),
        max_results: MAX_RESPONSE_RESULTS,
        count: truncated_children.len(),
        children: truncated_children,
    }))
}
