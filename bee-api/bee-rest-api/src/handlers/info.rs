// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig,
    handlers::{health::is_healthy, EnvelopeContent, SuccessEnvelope},
    storage::Backend,
    NetworkId,
};
use bee_common::node::ResHandle;
use bee_protocol::tangle::MsTangle;
use serde::Serialize;
use std::convert::Infallible;
use warp::Reply;

pub(crate) async fn info<B: Backend>(
    tangle: ResHandle<MsTangle<B>>,
    network_id: NetworkId,
    config: RestApiConfig,
) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&SuccessEnvelope::new(GetInfoResponse {
        name: String::from("Bee"),
        version: String::from(env!("CARGO_PKG_VERSION")),
        is_healthy: is_healthy(tangle.clone()).await,
        network_id: network_id.0,
        latest_milestone_index: *tangle.get_latest_milestone_index(),
        solid_milestone_index: *tangle.get_latest_milestone_index(),
        pruning_index: *tangle.get_pruning_index(),
        features: {
            let mut features = Vec::new();
            if config.allow_proof_of_work() {
                features.push("PoW".to_string())
            }
            features
        },
    })))
}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize)]
pub struct GetInfoResponse {
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
}

impl EnvelopeContent for GetInfoResponse {}
