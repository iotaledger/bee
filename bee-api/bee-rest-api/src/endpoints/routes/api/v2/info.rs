// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_block::constant::{PROTOCOL_VERSION, TOKEN_SUPPLY};

use crate::{
    endpoints::{routes::health, storage::StorageBackend, ApiArgsFullNode},
    types::responses::{
        BaseTokenResponse, ConfirmedMilestoneResponse, InfoResponse, LatestMilestoneResponse, MetricsResponse,
        ProtocolResponse, RentStructureResponse, StatusResponse,
    },
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/info", get(info::<B>))
}

async fn info<B: StorageBackend>(Extension(args): Extension<ApiArgsFullNode<B>>) -> impl IntoResponse {
    let (latest_milestone_index, latest_milestone_metadata) = {
        let latest_milestone_index = args.tangle.get_latest_milestone_index();
        (
            latest_milestone_index,
            args.tangle.get_milestone_metadata(latest_milestone_index),
        )
    };

    let (confirmed_milestone_index, confirmed_milestone_metadata) = {
        let confirmed_milestone_index = args.tangle.get_confirmed_milestone_index();
        (
            confirmed_milestone_index,
            args.tangle.get_milestone_metadata(confirmed_milestone_index),
        )
    };

    Json(InfoResponse {
        name: args.node_info.name.clone(),
        version: args.node_info.version.clone(),
        status: StatusResponse {
            is_healthy: health::is_healthy(&args.tangle, &args.peer_manager),
            // TODO: In future, the snapshot might make all data for the `latest_milestone` available.
            latest_milestone: LatestMilestoneResponse {
                index: *latest_milestone_index,
                timestamp: latest_milestone_metadata
                    .as_ref()
                    .map(|m| m.timestamp())
                    .unwrap_or_default(),
                milestone_id: latest_milestone_metadata
                    .map(|m| m.milestone_id().to_string())
                    .unwrap_or_default(),
            },
            // TODO: In future, the snapshot might make all data for the `confirmed_milestone` available.
            confirmed_milestone: ConfirmedMilestoneResponse {
                index: *confirmed_milestone_index,
                timestamp: confirmed_milestone_metadata
                    .as_ref()
                    .map(|m| m.timestamp())
                    .unwrap_or_default(),
                milestone_id: confirmed_milestone_metadata
                    .map(|m| m.milestone_id().to_string())
                    .unwrap_or_default(),
            },
            pruning_index: *args.tangle.get_pruning_index(),
        },
        protocol: ProtocolResponse {
            version: PROTOCOL_VERSION,
            network_name: args.network_name.clone(),
            bech32_hrp: args.bech32_hrp.clone(),
            min_pow_score: args.protocol_config.minimum_pow_score(),
            rent_structure: RentStructureResponse {
                v_byte_cost: args.protocol_config.byte_cost().v_byte_cost,
                v_byte_factor_key: args.protocol_config.byte_cost().v_byte_factor_key,
                v_byte_factor_data: args.protocol_config.byte_cost().v_byte_factor_data,
            },
            token_supply: TOKEN_SUPPLY.to_string(),
        },
        base_token: BaseTokenResponse {
            name: "Shimmer".to_string(), // TODO: don't hardcode
            ticker_symbol: "SMR".to_string(),
            unit: "SMR".to_string(),
            subunit: Some("glow".to_string()),
            decimals: 6,
            use_metric_prefix: false,
        },
        metrics: MetricsResponse {
            blocks_per_second: 0f64,            // TODO: use actual metrics values
            referenced_blocks_per_second: 0f64, // TODO: use actual metrics values
            referenced_rate: 0f64,              // TODO: use actual metrics values
        },
        features: {
            let mut features = Vec::new();
            if args.rest_api_config.feature_proof_of_work() {
                features.push("PoW".to_string())
            }
            features
        },
        plugins: Vec::new(), // TODO: add actual plugins that the node supports
    })
}
