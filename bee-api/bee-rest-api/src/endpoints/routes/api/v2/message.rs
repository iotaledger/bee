// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::{MessageDto, MessageId};

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::responses::MessageResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages/:message_id", get(message::<B>))
}

pub(crate) async fn message<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&message_id) {
        Some(message) => Ok(Json(MessageResponse(MessageDto::from(&message)))),
        None => Err(ApiError::NotFound),
    }
}
