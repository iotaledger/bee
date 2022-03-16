// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{config::ROUTE_ADD_PEER, storage::StorageBackend},
    types::{
        dtos::{PeerDto, RelationDto},
        responses::AddPeerResponse,
    },
};

use bee_gossip::{Command::AddPeer, Multiaddr, NetworkCommandSender, PeerId, PeerRelation, Protocol};
use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde_json::Value;

use std::net::IpAddr;

use crate::endpoints::{error::ApiError, ApiArgsFullNode};
use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::post,
    Router,
};
use std::sync::Arc;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers", post(add_peer::<B>))
}

pub(crate) async fn add_peer<B: StorageBackend>(
    Json(value): Json<Value>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let multi_address_v = &value["multiAddress"];
    let alias_v = &value["alias"];

    let mut multi_address = multi_address_v
        .as_str()
        .ok_or_else(|| ApiError::BadRequest("invalid multi address: expected a string".to_string()))?
        .parse::<Multiaddr>()
        .map_err(|e| ApiError::BadRequest(format!("invalid multi address: {}", e)))?;

    let peer_id = match multi_address.pop() {
        Some(Protocol::P2p(multihash)) => PeerId::from_multihash(multihash)
            .map_err(|_| ApiError::BadRequest("invalid multi address: can not parse peer id".to_string()))?,
        _ => {
            return Err(ApiError::BadRequest(
                "invalid multi address: invalid protocol type".to_string(),
            ));
        }
    };

    args.peer_manager
        .get_map(&peer_id, |peer_entry| {
            let peer_dto = PeerDto::from(peer_entry.0.as_ref());
            Ok(Json(AddPeerResponse(peer_dto)))
        })
        .unwrap_or_else(|| {
            let alias = if alias_v.is_null() {
                None
            } else {
                Some(
                    alias_v
                        .as_str()
                        .ok_or_else(|| ApiError::BadRequest("invalid alias: expected a string".to_string()))?
                        .to_string(),
                )
            };

            if let Err(e) = args.network_command_sender.send(AddPeer {
                peer_id,
                multiaddr: multi_address.clone(),
                alias: alias.clone(),
                relation: PeerRelation::Known,
            }) {
                return Err(ApiError::NotFound(format!("failed to add peer: {}", e)));
            }

            Ok(Json(AddPeerResponse(PeerDto {
                id: peer_id.to_string(),
                alias,
                multi_addresses: vec![multi_address.to_string()],
                relation: RelationDto::Known,
                connected: false,
                gossip: None,
            })))
        })
}
