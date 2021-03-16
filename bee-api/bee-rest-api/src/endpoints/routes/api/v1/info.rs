// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
    types::{body::SuccessBody, responses::InfoResponse},
};

use bee_protocol::{config::ProtocolConfig, PeerManager};
use bee_runtime::{node::NodeInfo, resource::ResourceHandle};
use bee_tangle::MsTangle;

use warp::{Filter, Rejection, Reply};

use std::{convert::Infallible, net::IpAddr};

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("info")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    node_info: ResourceHandle<NodeInfo>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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
        .and_then(info)
}

pub(crate) async fn info<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    node_info: ResourceHandle<NodeInfo>,
    peer_manager: ResourceHandle<PeerManager>,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&SuccessBody::new(InfoResponse {
        name: node_info.name.clone(),
        version: node_info.version.clone(),
        is_healthy: health::is_healthy(&tangle, &peer_manager).await,
        network_id: network_id.0,
        bech32_hrp,
        latest_milestone_index: *tangle.get_latest_milestone_index(),
        confirmed_milestone_index: *tangle.get_confirmed_milestone_index(),
        pruning_index: *tangle.get_pruning_index(),
        features: {
            let mut features = Vec::new();
            if rest_api_config.feature_proof_of_work() {
                features.push("PoW".to_string())
            }
            features
        },
        min_pow_score: protocol_config.minimum_pow_score(),
    })))
}
