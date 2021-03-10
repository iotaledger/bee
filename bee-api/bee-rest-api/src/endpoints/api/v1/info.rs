// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::{RestApiConfig, ROUTE_INFO},
    endpoints::health,
    filters::{
        with_bech32_hrp, 
        with_network_id, 
        with_node_info, 
        with_peer_manager, 
        with_protocol_config, 
        with_rest_api_config, 
        with_tangle
    },
    permission::has_permission,
    storage::StorageBackend,
    Bech32Hrp, NetworkId,
};

use bee_protocol::{config::ProtocolConfig, PeerManager};
use bee_runtime::{node::NodeInfo, resource::ResourceHandle};
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use std::{convert::Infallible, net::IpAddr};

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
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("info"))
        .and(warp::path::end())
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

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    #[serde(rename = "isHealthy")]
    pub is_healthy: bool,
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "bech32HRP")]
    pub bech32_hrp: String,
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    #[serde(rename = "confirmedMilestoneIndex")]
    pub confirmed_milestone_index: u32,
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
    pub features: Vec<String>,
    #[serde(rename = "minPowScore")]
    pub min_pow_score: f64,
}

impl BodyInner for InfoResponse {}
