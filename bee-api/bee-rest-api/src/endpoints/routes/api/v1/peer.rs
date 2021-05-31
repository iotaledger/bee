// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_PEER, filters::with_peer_manager, path_params::peer_id, permission::has_permission,
        rejection::CustomRejection,
    },
    types::{body::SuccessBody, dtos::PeerDto, responses::PeerResponse},
};

use bee_network::PeerId;
use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;

use warp::{reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("peers"))
        .and(peer_id())
        .and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_PEER, public_routes, allowed_ips))
        .and(with_peer_manager(peer_manager))
        .and_then(peer)
}

pub(crate) async fn peer(peer_id: PeerId, peer_manager: ResourceHandle<PeerManager>) -> Result<impl Reply, Rejection> {
    match peer_manager.get(&peer_id).await {
        Some(peer_entry) => Ok(warp::reply::json(&SuccessBody::new(PeerResponse(PeerDto::from(
            peer_entry.0.as_ref(),
        ))))),
        None => Err(reject::custom(CustomRejection::NotFound("peer not found".to_string()))),
    }
}
