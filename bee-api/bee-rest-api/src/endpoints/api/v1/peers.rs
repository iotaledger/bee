// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_PEERS,
    filters::with_peer_manager,
    permission::has_permission,
    types::{peer_to_peer_dto, PeerDto},
};

use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::{Filter, Reply, Rejection};

use std::{convert::Infallible, net::IpAddr};

pub(crate) fn peers_filter(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("peers"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_PEERS, public_routes, allowed_ips))
        .and(with_peer_manager(peer_manager))
        .and_then(peers)
}

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
