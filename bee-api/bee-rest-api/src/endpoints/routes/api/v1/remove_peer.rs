// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_REMOVE_PEER, filters::with_gossip_command_tx, path_params::peer_id, permission::has_permission,
    rejection::CustomRejection,
};

use bee_gossip::{GossipManagerCommand::RemovePeer, GossipManagerCommandTx, PeerId};
use bee_runtime::resource::ResourceHandle;

use warp::{filters::BoxedFilter, http::StatusCode, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (PeerId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("peers"))
        .and(peer_id())
        .and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    gossip_command_tx: ResourceHandle<GossipManagerCommandTx>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::delete())
        .and(has_permission(ROUTE_REMOVE_PEER, public_routes, allowed_ips))
        .and(with_gossip_command_tx(gossip_command_tx))
        .and_then(remove_peer)
        .boxed()
}

pub(crate) async fn remove_peer(
    peer_id: PeerId,
    gossip_command_tx: ResourceHandle<GossipManagerCommandTx>,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = gossip_command_tx.send(RemovePeer { peer_id }) {
        return Err(reject::custom(CustomRejection::NotFound(format!(
            "failed to remove peer: {}",
            e
        ))));
    }
    Ok(StatusCode::NO_CONTENT)
}
