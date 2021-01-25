// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    types::{GossipDto, PeerDto},
};

use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::Reply;

use std::convert::Infallible;
use crate::types::{RelationDto, HeartbeatDto, MetricsDto, peer_to_peer_dto};

pub(crate) async fn peers(peer_manager: ResourceHandle<PeerManager>) -> Result<impl Reply, Infallible> {
    let mut peer_dtos = Vec::new();
    for peer in peer_manager.get_all().await {
        let peer_dto = peer_to_peer_dto(peer, peer_manager);
        peer_dtos.push(peer_dto);
    }
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(peer_dtos))))
}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

impl BodyInner for PeersResponse {}
