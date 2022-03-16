// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{routes::health, storage::StorageBackend},
    types::responses::{InfoResponse, MetricsResponse, ProtocolResponse, RentStructureResponse, StatusResponse},
};

use crate::endpoints::ApiArgsFullNode;
use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/info", get(info::<B>))
}

pub(crate) async fn info<B: StorageBackend>(Extension(args): Extension<Arc<ApiArgsFullNode<B>>>) -> impl IntoResponse {
    let latest_milestone_index = args.tangle.get_latest_milestone_index();
    let latest_milestone_timestamp = args
        .tangle
        .get_milestone(latest_milestone_index)
        .await
        .map(|m| m.timestamp())
        .unwrap_or_default();

    Json(InfoResponse {
        name: args.node_info.name.clone(),
        version: args.node_info.version.clone(),
        status: StatusResponse {
            is_healthy: health::is_healthy(&args.tangle, &args.peer_manager).await,
            latest_milestone_timestamp,
            latest_milestone_index: *latest_milestone_index,
            confirmed_milestone_index: *args.tangle.get_confirmed_milestone_index(),
            pruning_index: *args.tangle.get_pruning_index(),
        },
        protocol: ProtocolResponse {
            network_name: args.network_id.0.clone(),
            bech32_hrp: args.bech32_hrp.clone(),
            min_pow_score: args.protocol_config.minimum_pow_score(),
            rent_structure: RentStructureResponse {
                v_byte_cost: args.protocol_config.byte_cost().v_byte_cost,
                v_byte_factor_key: args.protocol_config.byte_cost().v_byte_factor_key,
                v_byte_factor_data: args.protocol_config.byte_cost().v_byte_factor_data,
            },
        },
        metrics: MetricsResponse {
            messages_per_second: 0.0,            // TODO
            referenced_messages_per_second: 0.0, // TODO
            referenced_rate: 0.0,                // TODO
        },
        features: {
            let mut features = Vec::new();
            if args.rest_api_config.feature_proof_of_work() {
                features.push("PoW".to_string())
            }
            features
        },
        plugins: Vec::new(), // TODO
    })
}
