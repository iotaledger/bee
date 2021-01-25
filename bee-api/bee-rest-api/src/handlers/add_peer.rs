// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    types::{GossipMetricsDto, PeerDto},
};

use bee_network::{PeerId, NetworkController, PeerRelation};
use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use warp::{reject, Rejection, Reply};

use crate::filters::CustomRejection::NotFound;

pub(crate) async fn add_peer(value: JsonValue, network_controller: ResourceHandle<NetworkController>) -> Result<impl Reply, Rejection> {

    if let Err(e) = network.send(AddPeer {
        id,
        address,
        alias,
        relation: PeerRelation::Known,
    }) {
        warn!("Failed to add peer: {}", e);
    }

}

/// Response of GET /api/v1/peer/{peer_id}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerResponse(pub PeerDto);

impl BodyInner for PeerResponse {}
