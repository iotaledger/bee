// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    rejection::CustomRejection,
    types::peer_to_peer_dto,
    types::PeerDto,
};

use bee_network::PeerId;
use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

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
