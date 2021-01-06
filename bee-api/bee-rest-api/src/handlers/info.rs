// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig,
    handlers::{health::is_healthy, BodyInner, SuccessBody},
    storage::StorageBackend,
    NetworkId,
};

use bee_common_pt2::node::ResHandle;
use bee_protocol::{config::ProtocolConfig, tangle::MsTangle};

use serde::Serialize;
use warp::Reply;

use std::convert::Infallible;

pub(crate) async fn info<B: StorageBackend>(
    tangle: ResHandle<MsTangle<B>>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&SuccessBody::new(InfoResponse {
        name: String::from("Bee"),
        version: String::from(env!("CARGO_PKG_VERSION")),
        is_healthy: is_healthy(tangle.clone()).await,
        network_id: network_id.0,
        latest_milestone_index: *tangle.get_latest_milestone_index(),
        solid_milestone_index: *tangle.get_latest_solid_milestone_index(),
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
#[derive(Clone, Debug, Serialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    #[serde(rename = "isHealthy")]
    pub is_healthy: bool,
    #[serde(rename = "networkId")]
    pub network_id: String,
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    #[serde(rename = "solidMilestoneIndex")]
    pub solid_milestone_index: u32,
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
    pub features: Vec<String>,
    #[serde(rename = "minPowScore")]
    pub min_pow_score: f64,
}

impl BodyInner for InfoResponse {}
