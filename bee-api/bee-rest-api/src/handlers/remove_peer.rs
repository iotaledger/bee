// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_network::{Command::RemovePeer, NetworkController, PeerId};
use bee_runtime::resource::ResourceHandle;

use warp::{http::StatusCode, reject, Rejection, Reply};

use crate::filters::CustomRejection::NotFound;

pub(crate) async fn remove_peer(
    peer_id: PeerId,
    network_controller: ResourceHandle<NetworkController>,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = network_controller.send(RemovePeer { peer_id }) {
        return Err(reject::custom(NotFound(format!("failed to remove peer: {}", e))));
    }
    Ok(StatusCode::OK)
}
