// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_TIPS, permission::has_permission, rejection::CustomRejection,
        storage::StorageBackend, CONFIRMED_THRESHOLD,
    },
    types::responses::TipsResponse,
};

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
        .route("/tips", get(tips::<B>))
}

pub(crate) async fn tips<B: StorageBackend>(Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,) -> Result<impl IntoResponse, ApiError> {
    if !args.tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(ApiError::ServiceUnavailable(
            "the node is not synchronized".to_string(),
        ));
    }
    match args.tangle.get_messages_to_approve().await {
        Some(tips) => Ok(Json(TipsResponse {
            tip_message_ids: tips.iter().map(|t| t.to_string()).collect(),
        })),
        None => Err(ApiError::ServiceUnavailable(
            "tip pool is empty".to_string(),
        )),
    }
}
