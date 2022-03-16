// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE,  path_params::milestone_index, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::responses::MilestoneResponse,
};

use bee_message::milestone::MilestoneIndex;
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
        .route("/milestones/:milestone_index", get(milestone::<B>))
}

pub(crate) async fn milestone<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    match args.tangle.get_milestone_message_id(milestone_index).await {
        Some(message_id) => match args.tangle.get_metadata(&message_id).await {
            Some(metadata) => Ok(Json(MilestoneResponse {
                milestone_index: *milestone_index,
                message_id: message_id.to_string(),
                timestamp: metadata.arrival_timestamp(),
            })),
            None => Err(ApiError::NotFound(
                "can not find metadata for milestone".to_string(),
            )),
        },
        None => Err(ApiError::NotFound(
            "can not find milestone".to_string(),
        )),
    }
}
