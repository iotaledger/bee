// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::{Command::AddPeer, Multiaddr, PeerId, PeerRelation, Protocol};
use serde_json::Value as JsonValue;
use warp::{filters::BoxedFilter, http::StatusCode, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{filters::with_args, rejection::CustomRejection, storage::StorageBackend, ApiArgsFullNode},
    types::{
        body::SuccessBody,
        dtos::{PeerDto, RelationDto},
        responses::AddPeerResponse,
    },
};

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("peers")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::post())
        .and(warp::body::json())
        .and(with_args(args))
        .and_then(|value, args| async move { add_peer(value, args) })
        .boxed()
}

pub(crate) fn add_peer<B: StorageBackend>(value: JsonValue, args: ApiArgsFullNode<B>) -> Result<impl Reply, Rejection> {
    let multi_address_v = &value["multiAddress"];
    let alias_v = &value["alias"];

    let mut multi_address = multi_address_v
        .as_str()
        .ok_or_else(|| {
            reject::custom(CustomRejection::BadRequest(
                "invalid multi address: expected a string".to_string(),
            ))
        })?
        .parse::<Multiaddr>()
        .map_err(|e| reject::custom(CustomRejection::BadRequest(format!("invalid multi address: {}", e))))?;

    let peer_id = match multi_address.pop() {
        Some(Protocol::P2p(multihash)) => PeerId::from_multihash(multihash).map_err(|_| {
            reject::custom(CustomRejection::BadRequest(
                "invalid multi address: can not parse peer id".to_string(),
            ))
        })?,
        _ => {
            return Err(reject::custom(CustomRejection::BadRequest(
                "invalid multi address: invalid protocol type".to_string(),
            )));
        }
    };

    args.peer_manager
        .get_map(&peer_id, |peer_entry| {
            let peer_dto = PeerDto::from(peer_entry.0.as_ref());
            Ok(warp::reply::with_status(
                warp::reply::json(&SuccessBody::new(AddPeerResponse(peer_dto))),
                StatusCode::OK,
            ))
        })
        .unwrap_or_else(|| {
            let alias = if alias_v.is_null() {
                None
            } else {
                Some(
                    alias_v
                        .as_str()
                        .ok_or_else(|| {
                            reject::custom(CustomRejection::BadRequest(
                                "invalid alias: expected a string".to_string(),
                            ))
                        })?
                        .to_string(),
                )
            };

            if let Err(e) = args.network_command_sender.send(AddPeer {
                peer_id,
                multiaddr: multi_address.clone(),
                alias: alias.clone(),
                relation: PeerRelation::Known,
            }) {
                return Err(reject::custom(CustomRejection::NotFound(format!(
                    "failed to add peer: {}",
                    e
                ))));
            }

            Ok(warp::reply::with_status(
                warp::reply::json(&SuccessBody::new(AddPeerResponse(PeerDto {
                    id: peer_id.to_string(),
                    alias,
                    multi_addresses: vec![multi_address.to_string()],
                    relation: RelationDto::Known,
                    connected: false,
                    gossip: None,
                }))),
                StatusCode::OK,
            ))
        })
}
