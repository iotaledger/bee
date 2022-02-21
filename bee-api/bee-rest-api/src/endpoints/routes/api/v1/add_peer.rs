// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_ADD_PEER,
        filters::{with_network_command_sender, with_peer_manager},
        permission::has_permission,
        rejection::CustomRejection,
    },
    types::{
        body::SuccessBody,
        dtos::{PeerDto, RelationDto},
        responses::AddPeerResponse,
    },
};

use bee_gossip::{Command::AddPeer, Multiaddr, NetworkCommandSender, PeerId, PeerRelation, Protocol};
use bee_protocol::workers::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde_json::Value as JsonValue;
use warp::{filters::BoxedFilter, http::StatusCode, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("peers")).and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    peer_manager: ResourceHandle<PeerManager>,
    network_command_sender: ResourceHandle<NetworkCommandSender>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::post())
        .and(has_permission(ROUTE_ADD_PEER, public_routes, allowed_ips))
        .and(warp::body::json())
        .and(with_peer_manager(peer_manager))
        .and(with_network_command_sender(network_command_sender))
        .and_then(
            |value, peer_manager, network_controller| async move { add_peer(value, peer_manager, network_controller) },
        )
        .boxed()
}

pub(crate) fn add_peer(
    value: JsonValue,
    peer_manager: ResourceHandle<PeerManager>,
    network_controller: ResourceHandle<NetworkCommandSender>,
) -> Result<impl Reply, Rejection> {
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

    peer_manager
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

            if let Err(e) = network_controller.send(AddPeer {
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
