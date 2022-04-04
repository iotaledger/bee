// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::MessageId;

use crate::{
    endpoints::{error::ApiError, routes::api::v2::MAX_RESPONSE_RESULTS, storage::StorageBackend, ApiArgsFullNode},
    types::responses::MessageChildrenResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages/:message_id/children", get(message_children::<B>))
}

pub(crate) async fn message_children<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let mut children = Vec::from_iter(args.tangle.get_children(&message_id).await.unwrap_or_default());
    let count = children.len();
    children.truncate(MAX_RESPONSE_RESULTS);
    Ok(Json(MessageChildrenResponse {
        message_id: message_id.to_string(),
        max_results: MAX_RESPONSE_RESULTS,
        count,
        children_message_ids: children.iter().map(MessageId::to_string).collect(),
    }))
}
