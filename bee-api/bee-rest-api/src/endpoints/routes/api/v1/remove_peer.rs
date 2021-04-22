// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_REMOVE_PEER, filters::with_network_command_sender, path_params::peer_id, permission::has_permission,
    rejection::CustomRejection,
};

use bee_network::{Command::RemovePeer, NetworkCommandSender, PeerId};
use bee_runtime::resource::ResourceHandle;

use warp::{http::StatusCode, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (PeerId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("peers"))
        .and(peer_id())
        .and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    network_command_sender: ResourceHandle<NetworkCommandSender>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::delete())
        .and(has_permission(ROUTE_REMOVE_PEER, public_routes, allowed_ips))
        .and(with_network_command_sender(network_command_sender))
        .and_then(remove_peer)
}

pub(crate) async fn remove_peer(
    peer_id: PeerId,
    network_controller: ResourceHandle<NetworkCommandSender>,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = network_controller.send(RemovePeer { peer_id }) {
        return Err(reject::custom(CustomRejection::NotFound(format!(
            "failed to remove peer: {}",
            e
        ))));
    }
    Ok(StatusCode::NO_CONTENT)
}
