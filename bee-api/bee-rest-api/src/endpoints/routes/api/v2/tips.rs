// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_message::MessageId;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode, CONFIRMED_THRESHOLD},
    types::responses::TipsResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/tips", get(tips::<B>))
}

pub(crate) async fn tips<B: StorageBackend>(
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    if !args.tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(ApiError::ServiceUnavailable("the node is not synchronized".to_string()));
    }
    match args.tangle.get_messages_to_approve().await {
        Some(tips) => Ok(Json(TipsResponse {
            tip_message_ids: tips.iter().map(MessageId::to_string).collect(),
        })),
        None => Err(ApiError::ServiceUnavailable("tip pool is empty".to_string())),
    }
}
