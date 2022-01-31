// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{config::ROUTE_PEERS, filters::with_peer_manager, permission::has_permission},
    types::{body::SuccessBody, dtos::PeerDto, responses::PeersResponse},
};

use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::{convert::Infallible, net::IpAddr};

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("peers")).and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    peer_manager: ResourceHandle<PeerManager>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_PEERS, public_routes, allowed_ips))
        .and(with_peer_manager(peer_manager))
        .and_then(|peer_manager| async move { peers(peer_manager) })
        .boxed()
}

pub(crate) fn peers(peer_manager: ResourceHandle<PeerManager>) -> Result<impl Reply, Infallible> {
    let mut peers_dtos = Vec::new();
    for peer in peer_manager.get_all() {
        peers_dtos.push(PeerDto::from(peer.as_ref()));
    }
    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(peers_dtos))))
}
