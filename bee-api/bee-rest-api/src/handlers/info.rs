// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig,
    constants::{BEE_GIT_COMMIT, BEE_VERSION},
    handlers::{BodyInner, SuccessBody},
    storage::StorageBackend,
    Bech32Hrp, NetworkId,
};

use bee_protocol::config::ProtocolConfig;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use serde::{Deserialize, Serialize};
use warp::Reply;

use std::convert::Infallible;

pub(crate) async fn info<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&SuccessBody::new(InfoResponse {
        name: String::from("Bee"),
        version: {
            let version = if BEE_GIT_COMMIT.is_empty() {
                BEE_VERSION.to_owned()
            } else {
                BEE_VERSION.to_owned() + "-" + &BEE_GIT_COMMIT[0..7]
            };
            version
        },
        is_healthy: tangle.is_healthy().await,
        network_id: network_id.0,
        bech32_hrp,
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
    #[serde(rename = "solidMilestoneIndex")]
    pub solid_milestone_index: u32,
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
    pub features: Vec<String>,
    #[serde(rename = "minPowScore")]
    pub min_pow_score: f64,
}

impl BodyInner for InfoResponse {}
