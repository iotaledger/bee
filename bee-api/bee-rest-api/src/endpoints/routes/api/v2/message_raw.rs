// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::MessageId;
use packable::PackableExt;

use crate::endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages/:message_id/raw", get(message_raw::<B>))
}

pub(crate) async fn message_raw<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&message_id).await.map(|m| (*m).clone()) {
        Some(message) => Ok(message.pack_to_vec()),
        None => Err(ApiError::NotFound("can not find message".to_string())),
    }
}
