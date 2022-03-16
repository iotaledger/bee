// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MESSAGE_CHILDREN,  path_params::message_id, permission::has_permission,
        storage::StorageBackend,
    },
    types::responses::MessageChildrenResponse,
};

use bee_message::MessageId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::net::IpAddr;

use axum::extract::Extension;
use crate::endpoints::ApiArgsFullNode;
use axum::extract::Json;
use axum::Router;
use axum::routing::get;
use axum::response::IntoResponse;
use crate::endpoints::error::ApiError;
use std::sync::Arc;
use axum::extract::Path;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new()
        .route("/messages/:message_id/children", get(message_children::<B>))
}

pub(crate) async fn message_children<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let mut children = Vec::from_iter(args.tangle.get_children(&message_id).await.unwrap_or_default());
    let count = children.len();
    let max_results = 1000;
    children.truncate(max_results);
    Ok(Json(MessageChildrenResponse {
        message_id: message_id.to_string(),
        max_results,
        count,
        children_message_ids: children.iter().map(|id| id.to_string()).collect(),
    }))
}
