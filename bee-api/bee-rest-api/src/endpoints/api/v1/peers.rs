// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    types::{peer_to_peer_dto, PeerDto},
};

use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::Reply;

use std::convert::Infallible;

pub(crate) async fn peers(peer_manager: ResourceHandle<PeerManager>) -> Result<impl Reply, Infallible> {
    let mut peers_dtos = Vec::new();
    for peer in peer_manager.get_all().await {
        peers_dtos.push(peer_to_peer_dto(&peer, &peer_manager).await);
    }
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(peers_dtos))))
}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

impl BodyInner for PeersResponse {}
