// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{config::ROUTE_MESSAGE, storage::StorageBackend},
    types::{dtos::MessageDto, responses::MessageResponse},
};

use bee_message::MessageId;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;

use std::net::IpAddr;

use crate::endpoints::{error::ApiError, ApiArgsFullNode};
use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages/:message_id", get(message::<B>))
}

pub(crate) async fn message<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => Ok(Json(MessageResponse(MessageDto::from(&message)))),
        None => Err(ApiError::NotFound("can not find message".to_string())),
    }
}
