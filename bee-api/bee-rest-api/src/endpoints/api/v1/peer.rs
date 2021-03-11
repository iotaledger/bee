// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::{BodyInner, SuccessBody},
    config::ROUTE_PEER,
    filters::with_peer_manager,
    path_params::peer_id,
    permission::has_permission,
    rejection::CustomRejection,
    types::{peer_to_peer_dto, PeerDto},
};

use bee_network::PeerId;
use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::{Filter, reject, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("peers"))
        .and(peer_id())
        .and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
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
        Some(peer_entry) => Ok(warp::reply::json(&SuccessBody::new(PeerResponse(
            peer_to_peer_dto(&peer_entry.0, &peer_manager).await,
        )))),
        None => Err(reject::custom(CustomRejection::NotFound("peer not found".to_string()))),
    }
}

/// Response of GET /api/v1/peer/{peer_id}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerResponse(pub PeerDto);

impl BodyInner for PeerResponse {}
