// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{config::ROUTE_PEERS, filters::with_peer_manager, permission::has_permission},
    types::{body::SuccessBody, dtos::PeerDto, responses::PeersResponse},
};

use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;

use warp::{Filter, Rejection, Reply};

use std::{convert::Infallible, net::IpAddr};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("peers")).and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_PEERS, public_routes, allowed_ips))
        .and(with_peer_manager(peer_manager))
        .and_then(peers)
}

pub(crate) async fn peers(peer_manager: ResourceHandle<PeerManager>) -> Result<impl Reply, Infallible> {
    let mut peers_dtos = Vec::new();
    for peer in peer_manager.get_all().await {
        peers_dtos.push(PeerDto::from(peer.as_ref()));
    }
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(peers_dtos))))
}
