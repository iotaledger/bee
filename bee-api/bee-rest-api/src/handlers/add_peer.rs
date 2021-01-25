// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    types::{GossipDto, PeerDto},
};

use bee_network::{Command::AddPeer, Multiaddr, NetworkController, PeerId, PeerRelation, Protocol};
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use warp::{reject, Rejection, Reply};

use crate::filters::CustomRejection::{BadRequest, NotFound};
use bee_protocol::PeerManager;
use warp::http::StatusCode;
use crate::types::{RelationDto, HeartbeatDto, MetricsDto, peer_to_peer_dto};

pub(crate) async fn add_peer(
    value: JsonValue,
    peer_manager: ResourceHandle<PeerManager>,
    network_controller: ResourceHandle<NetworkController>,
) -> Result<impl Reply, Rejection> {
    let multi_address_v = &value["multiAddress"];
    let alias_v = &value["alias"];

    let mut multi_address = multi_address_v
        .as_str()
        .ok_or_else(|| reject::custom(BadRequest("invalid multi address: expected a string".to_string())))?
        .parse::<Multiaddr>()
        .map_err(|e| reject::custom(BadRequest(format!("invalid multi address: {}", e))))?;

    let peer_id = match multi_address.pop().unwrap() {
        Protocol::P2p(multihash) => PeerId::from_multihash(multihash)
            .map_err(|_| reject::custom(BadRequest("invalid multi address".to_string())))?,
        _ => {
            return Err(reject::custom(BadRequest(
                "Invalid peer descriptor. The multi address did not have a valid peer id as its last segment."
                    .to_string(),
            )))
        }
    };

    match peer_manager.get(&peer_id).await {
        Some(peer_entry) => {
            let peer_dto = peer_to_peer_dto(&peer_entry.0, peer_manager);
            Ok(warp::reply::with_status(
                warp::reply::json(&SuccessBody::new(AddPeerResponse(peer_dto))),
                StatusCode::OK,
            ))
        }

        None => {
            let alias = if alias_v.is_null() {
                None
            } else {
                Some(
                    alias_v
                        .as_str()
                        .ok_or_else(|| reject::custom(BadRequest("invalid alias: expected a string".to_string())))?
                        .to_string(),
                )
            };

            if let Err(e) = network_controller.send(AddPeer {
                id: peer_id,
                address: multi_address.clone(),
                alias: alias.clone(),
                relation: PeerRelation::Known,
            }) {
                return Err(reject::custom(NotFound(format!("failed to add peer: {}", e))));
            }

            Ok(warp::reply::with_status(
                warp::reply::json(&SuccessBody::new(AddPeerResponse(PeerDto {
                    id: peer_id.to_string(),
                    alias,
                    multi_addresses: vec![multi_address.to_string()],
                    relation: "known".to_string(),
                    connected: false,
                    gossip_metrics: GossipDto::default(),
                }))),
                StatusCode::OK,
            ))
        }
    }
}

/// Response of POST /api/v1/peers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddPeerResponse(pub PeerDto);

impl BodyInner for AddPeerResponse {}
