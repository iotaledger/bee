// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_MILESTONE_UTXO_CHANGES,  path_params::milestone_index,
        permission::has_permission, rejection::CustomRejection, storage::StorageBackend,
    },
    types::responses::UtxoChangesResponse,
};

use bee_ledger::types::OutputDiff;
use bee_message::milestone::MilestoneIndex;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

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
        .route("/milestones/:milestone_index/utxo-changes", get(milestone_utxo_changes::<B>))
}

pub(crate) async fn milestone_utxo_changes<B: StorageBackend>(
    Path(milestone_index): Path<MilestoneIndex>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let fetched = Fetch::<MilestoneIndex, OutputDiff>::fetch(&*args.storage, &milestone_index)
        .map_err(|_| {
            ApiError::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            )
        })?
        .ok_or_else(|| {
            ApiError::NotFound(
                "can not find Utxo changes for given milestone index".to_string(),
            )
        })?;

    Ok(Json(UtxoChangesResponse {
        index: *milestone_index,
        created_outputs: fetched.created_outputs().iter().map(|id| id.to_string()).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(|id| id.to_string()).collect(),
    }))
}