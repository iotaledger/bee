// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Extension, Json},
    response::IntoResponse,
    routing::post,
    Router,
};
use bee_gossip::{Command::AddPeer, Multiaddr, PeerId, PeerRelation, Protocol};
use log::error;
use serde_json::Value;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::{
        dtos::{PeerDto, RelationDto},
        responses::AddPeerResponse,
    },
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/peers", post(add_peer::<B>))
}

pub(crate) async fn add_peer<B: StorageBackend>(
    Json(value): Json<Value>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<impl IntoResponse, ApiError> {
    let multi_address_json = &value["multiAddress"];
    let alias_json = &value["alias"];

    let mut multi_address = multi_address_json
        .as_str()
        .ok_or_else(|| ApiError::BadRequest("invalid multiaddress"))?
        .parse::<Multiaddr>()
        .map_err(|_| ApiError::BadRequest("invalid multiaddress"))?;

    let peer_id = match multi_address.pop() {
        Some(Protocol::P2p(multihash)) => PeerId::from_multihash(multihash)
            .map_err(|_| ApiError::BadRequest("invalid multiaddress: can not parse peer id"))?,
        _ => {
            return Err(ApiError::BadRequest("invalid multi address: invalid protocol"));
        }
    };

    args.peer_manager
        .get_map(&peer_id, |peer_entry| {
            let peer_dto = PeerDto::from(peer_entry.0.as_ref());
            Ok(Json(AddPeerResponse(peer_dto)))
        })
        .unwrap_or_else(|| {
            let alias = if alias_json.is_null() {
                None
            } else {
                Some(
                    alias_json
                        .as_str()
                        .ok_or_else(|| ApiError::BadRequest("invalid alias: expected a string"))?
                        .to_string(),
                )
            };

            if let Err(e) = args.network_command_sender.send(AddPeer {
                peer_id,
                multiaddr: multi_address.clone(),
                alias: alias.clone(),
                relation: PeerRelation::Known,
            }) {
                error!("cannot add peer: {}", e);
                return Err(ApiError::InternalError);
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
