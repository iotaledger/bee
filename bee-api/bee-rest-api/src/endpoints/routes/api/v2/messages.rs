// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    http::header::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use bee_message::{MessageDto, MessageId};
use lazy_static::lazy_static;
use packable::PackableExt;

use crate::{
    endpoints::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode},
    types::responses::MessageResponse,
};

lazy_static! {
    pub(crate) static ref BYTE_CONTENT_HEADER: HeaderValue =
        HeaderValue::from_str("application/vnd.iota.serializer-v1").unwrap();
}

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/messages/:message_id", get(messages::<B>))
}

async fn messages<B: StorageBackend>(
    headers: HeaderMap,
    CustomPath(message_id): CustomPath<MessageId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<Response, ApiError> {
    if let Some(value) = headers.get(axum::http::header::CONTENT_TYPE) {
        if value.eq(&*BYTE_CONTENT_HEADER) {
            return messages_raw::<B>(message_id, args.clone())
                .await
                .map(|r| r.into_response());
        } else {
            messages_json::<B>(message_id, args.clone())
                .await
                .map(|r| r.into_response())
        }
    } else {
        messages_json::<B>(message_id, args.clone())
            .await
            .map(|r| r.into_response())
    }
}

pub(crate) async fn messages_json<B: StorageBackend>(
    message_id: MessageId,
    args: ApiArgsFullNode<B>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&message_id) {
        Some(message) => Ok(Json(MessageResponse(MessageDto::from(&message)))),
        None => Err(ApiError::NotFound),
    }
}

async fn messages_raw<B: StorageBackend>(
    message_id: MessageId,
    args: ApiArgsFullNode<B>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get(&message_id) {
        Some(message) => Ok(message.pack_to_vec()),
        None => Err(ApiError::NotFound),
    }
}
