// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{convert::Infallible, net::IpAddr};

use bee_message::constant::{PROTOCOL_VERSION, TOKEN_SUPPLY};
use bee_protocol::workers::{config::ProtocolConfig, PeerManager};
use bee_runtime::{node::NodeInfo, resource::ResourceHandle};
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::{
    endpoints::{
        config::{RestApiConfig, ROUTE_INFO},
        filters::{
            with_bech32_hrp, with_network_id, with_node_info, with_peer_manager, with_protocol_config,
            with_rest_api_config, with_tangle,
        },
        permission::has_permission,
        routes::health,
        storage::StorageBackend,
        Bech32Hrp, NetworkId,
    },
    types::responses::{
        BaseTokenResponse, ConfirmedMilestoneResponse, InfoResponse, LatestMilestoneResponse, MetricsResponse,
        ProtocolResponse, RentStructureResponse, StatusResponse,
    },
};

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("info")).and(warp::path::end())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    node_info: ResourceHandle<NodeInfo>,
    peer_manager: ResourceHandle<PeerManager>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_INFO, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and(with_network_id(network_id))
        .and(with_bech32_hrp(bech32_hrp))
        .and(with_rest_api_config(rest_api_config))
        .and(with_protocol_config(protocol_config))
        .and(with_node_info(node_info))
        .and(with_peer_manager(peer_manager))
        .and_then(
            |tangle, network_id, bech32_hrp, rest_api_config, protocol_config, node_info, peer_manager| async {
                info(
                    tangle,
                    network_id,
                    bech32_hrp,
                    rest_api_config,
                    protocol_config,
                    node_info,
                    peer_manager,
                )
            },
        )
        .boxed()
}

pub(crate) fn info<B: StorageBackend>(
    tangle: ResourceHandle<Tangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    node_info: ResourceHandle<NodeInfo>,
    peer_manager: ResourceHandle<PeerManager>,
) -> Result<impl Reply, Infallible> {
    let latest_milestone_index = tangle.get_latest_milestone_index();
    let confirmed_milestone_index = tangle.get_confirmed_milestone_index();

    Ok(warp::reply::json(&InfoResponse {
        name: node_info.name.clone(),
        version: node_info.version.clone(),
        status: StatusResponse {
            is_healthy: health::is_healthy(&tangle, &peer_manager),
            latest_milestone: LatestMilestoneResponse {
                index: *latest_milestone_index,
                timestamp: tangle
                    .get_milestone(latest_milestone_index)
                    .map(|m| m.timestamp())
                    .unwrap_or_default(),
                milestone_id: "".to_string(), // TODO: replace with milestone id using milestone id mapping
            },
            confirmed_milestone: ConfirmedMilestoneResponse {
                index: *confirmed_milestone_index,
                timestamp: tangle
                    .get_milestone(confirmed_milestone_index)
                    .map(|m| m.timestamp())
                    .unwrap_or_default(),
                milestone_id: "".to_string(), // TODO: replace with milestone id using milestone id mapping
            },
            pruning_index: *tangle.get_pruning_index(),
        },
        protocol: ProtocolResponse {
            version: PROTOCOL_VERSION,
            network_name: network_id.0,
            bech32_hrp,
            min_pow_score: protocol_config.minimum_pow_score(),
            rent_structure: RentStructureResponse {
                v_byte_cost: protocol_config.byte_cost().v_byte_cost,
                v_byte_factor_key: protocol_config.byte_cost().v_byte_factor_key,
                v_byte_factor_data: protocol_config.byte_cost().v_byte_factor_data,
            },
            token_supply: TOKEN_SUPPLY.to_string(),
        },
        base_token: BaseTokenResponse {
            name: "Shimmer".to_string(),
            #[serde(rename = "tickerSymbol")]
            ticker_symbol: "SMR".to_string(),
            unit: "SMR".to_string(),
            decimals: 6,
            subunit: "glow".to_string(),
            #[serde(rename = "useMetricPrefix")]
            use_metric_prefix: false,
        },
        metrics: MetricsResponse {
            messages_per_second: 0.0,            // TODO
            referenced_messages_per_second: 0.0, // TODO
            referenced_rate: 0.0,                // TODO
        },
        features: {
            let mut features = Vec::new();
            if rest_api_config.feature_proof_of_work() {
                features.push("PoW".to_string())
            }
            features
        },
        plugins: Vec::new(), // TODO
    }))
}
