// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
    Router::new().route("/messages/:message_id/raw", get(messages_raw::<B>))
}

pub(crate) async fn messages_raw<B: StorageBackend>(
    Path(message_id): Path<MessageId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&message_id) {
        Some(message) => Ok(message.pack_to_vec()),
        None => Err(ApiError::NotFound),
    }
}
