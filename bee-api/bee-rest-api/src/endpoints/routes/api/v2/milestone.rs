// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{config::ROUTE_MILESTONE, storage::StorageBackend},
    types::responses::MilestoneResponse,
};

use bee_message::milestone::MilestoneIndex;
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
    Router::new().route("/milestones/:milestone_index", get(milestone::<B>))
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
            None => Err(ApiError::NotFound("can not find metadata for milestone".to_string())),
        },
        None => Err(ApiError::NotFound("can not find milestone".to_string())),
    }
}
