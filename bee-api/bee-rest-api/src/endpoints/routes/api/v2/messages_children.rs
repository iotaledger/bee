// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::MessageId;

use crate::{
    endpoints::{
        error::ApiError, extractors::path::CustomPath, routes::api::v2::MAX_RESPONSE_RESULTS, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::responses::MessageChildrenResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages/:message_id/children", get(messages_children::<B>))
}

async fn messages_children<B: StorageBackend>(
    CustomPath(message_id): CustomPath<MessageId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let all_children = Vec::from_iter(args.tangle.get_children(&message_id).unwrap_or_default());

    let truncated_children = all_children
        .iter()
        .take(MAX_RESPONSE_RESULTS)
        .map(MessageId::to_string)
        .collect::<Vec<String>>();

    Ok(Json(MessageChildrenResponse {
        message_id: message_id.to_string(),
        max_results: MAX_RESPONSE_RESULTS,
        count: truncated_children.len(),
        children_message_ids: truncated_children,
    }))
}
