// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MESSAGE,  path_params::message_id, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::{dtos::MessageDto, responses::MessageResponse},
};

use bee_message::MessageId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

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
        .route("/messages/:message_id", get(message::<B>))
}

pub(crate) async fn message<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => Ok(Json(MessageResponse(MessageDto::from(&message)))),
        None => Err(ApiError::NotFound(
            "can not find message".to_string(),
        )),
    }
}
