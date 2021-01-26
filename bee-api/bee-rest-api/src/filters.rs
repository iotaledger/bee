// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig, filters::CustomRejection::BadRequest, handlers, storage::StorageBackend, Bech32Hrp,
    NetworkId,
};

use bee_network::{NetworkController, PeerId};
use bee_protocol::{config::ProtocolConfig, MessageSubmitterWorkerEvent, PeerManager};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use tokio::sync::mpsc;
use warp::{reject, Filter, Rejection};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) enum CustomRejection {
    BadRequest(String),
    NotFound(String),
    ServiceUnavailable(String),
}

impl reject::Reject for CustomRejection {}

pub fn all<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    storage: ResourceHandle<B>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    peer_manager: ResourceHandle<PeerManager>,
    network_controller: ResourceHandle<NetworkController>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    health(tangle.clone()).or(info(
        tangle.clone(),
        network_id.clone(),
        bech32_hrp,
        rest_api_config.clone(),
        protocol_config.clone(),
    )
    .or(tips(tangle.clone()))
    .or(submit_message(
        tangle.clone(),
        message_submitter.clone(),
        network_id,
        rest_api_config,
        protocol_config,
    ))
    .or(submit_message_raw(tangle.clone(), message_submitter))
    .or(message_indexation(storage.clone()))
    .or(message(tangle.clone()))
    .or(message_metadata(tangle.clone()))
    .or(message_raw(tangle.clone()))
    .or(message_children(tangle.clone()))
    .or(output(storage.clone()))
    .or(balance_bech32(storage.clone()))
    .or(balance_ed25519(storage.clone()))
    .or(outputs_bech32(storage.clone()))
    .or(outputs_ed25519(storage))
    .or(milestone(tangle))
    .or(peers(peer_manager.clone()))
    .or(peer_add(peer_manager.clone(), network_controller.clone()))
    .or(peer_remove(network_controller))
    .or(peer(peer_manager)))
}

fn health<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("health"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::health::health)
}

fn info<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and(with_network_id(network_id))
        .and(with_bech32_hrp(bech32_hrp))
        .and(with_rest_api_config(rest_api_config))
        .and(with_protocol_config(protocol_config))
        .and_then(handlers::info::info)
}

fn tips<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("tips"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::tips::tips)
}

fn submit_message<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::body::json())
        .and(with_tangle(tangle))
        .and(with_message_submitter(message_submitter))
        .and(with_network_id(network_id))
        .and(with_rest_api_config(rest_api_config))
        .and(with_protocol_config(protocol_config))
        .and_then(handlers::submit_message::submit_message)
}

fn submit_message_raw<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::body::bytes())
        .and(with_tangle(tangle))
        .and(with_message_submitter(message_submitter))
        .and_then(handlers::submit_message_raw::submit_message_raw)
}

fn message_indexation<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::query().and_then(|query: HashMap<String, String>| async move {
            match query.get("index") {
                Some(i) => Ok(i.to_string()),
                None => Err(reject::custom(BadRequest("invalid query parameter".to_string()))),
            }
        }))
        .and(with_storage(storage))
        .and_then(handlers::messages_find::messages_find)
}

fn message<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::message::message)
}

fn message_metadata<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("metadata"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::message_metadata::message_metadata)
}

fn message_raw<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("raw"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::message_raw::message_raw)
}

fn message_children<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(custom_path_param::message_id())
        .and(warp::path("children"))
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::message_children::message_children)
}

fn output<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("outputs"))
        .and(custom_path_param::output_id())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::output::output)
}

fn balance_bech32<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(custom_path_param::bech32_address())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::balance_bech32::balance_bech32)
}

fn balance_ed25519<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(custom_path_param::ed25519_address())
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::balance_ed25519::balance_ed25519)
}

fn outputs_bech32<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(custom_path_param::bech32_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::outputs_bech32::outputs_bech32)
}

fn outputs_ed25519<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(custom_path_param::ed25519_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
        .and(with_storage(storage))
        .and_then(handlers::outputs_ed25519::outputs_ed25519)
}

fn milestone<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(custom_path_param::milestone_index())
        .and(warp::path::end())
        .and(with_tangle(tangle))
        .and_then(handlers::milestone::milestone)
}

fn peers(
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("peers"))
        .and(warp::path::end())
        .and(with_peer_manager(peer_manager))
        .and_then(handlers::peers::peers)
}

fn peer(
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("peer"))
        .and(custom_path_param::peer_id())
        .and(warp::path::end())
        .and(with_peer_manager(peer_manager))
        .and_then(handlers::peer::peer)
}

fn peer_add(
    peer_manager: ResourceHandle<PeerManager>,
    network_controller: ResourceHandle<NetworkController>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("peer"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_peer_manager(peer_manager))
        .and(with_network_controller(network_controller))
        .and_then(handlers::add_peer::add_peer)
}

fn peer_remove(
    network_controller: ResourceHandle<NetworkController>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::delete()
        .and(warp::path("api"))
        .and(warp::path("v1"))
        .and(warp::path("peer"))
        .and(custom_path_param::peer_id())
        .and(warp::path::end())
        .and(with_network_controller(network_controller))
        .and_then(handlers::remove_peer::remove_peer)
}

mod custom_path_param {

    use super::*;
    use bee_message::{
        milestone::MilestoneIndex,
        payload::transaction::{Address, OutputId},
        prelude::Ed25519Address,
        MessageId,
    };

    pub(super) fn output_id() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<OutputId>() {
                Ok(id) => Ok(id),
                Err(_) => Err(reject::custom(BadRequest("invalid output id".to_string()))),
            }
        })
    }

    pub(super) fn message_id() -> impl Filter<Extract = (MessageId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<MessageId>() {
                Ok(msg) => Ok(msg),
                Err(_) => Err(reject::custom(BadRequest("invalid message id".to_string()))),
            }
        })
    }

    pub(super) fn milestone_index() -> impl Filter<Extract = (MilestoneIndex,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<u32>() {
                Ok(i) => Ok(MilestoneIndex(i)),
                Err(_) => Err(reject::custom(BadRequest("invalid milestone index".to_string()))),
            }
        })
    }

    pub(super) fn bech32_address() -> impl Filter<Extract = (Address,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match Address::try_from_bech32(&value) {
                Ok(addr) => Ok(addr),
                Err(_) => Err(reject::custom(BadRequest("invalid address".to_string()))),
            }
        })
    }

    pub(super) fn ed25519_address() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<Ed25519Address>() {
                Ok(addr) => Ok(addr),
                Err(_) => Err(reject::custom(BadRequest("invalid Ed25519 address".to_string()))),
            }
        })
    }

    pub(super) fn peer_id() -> impl Filter<Extract = (PeerId,), Error = Rejection> + Copy {
        warp::path::param().and_then(|value: String| async move {
            match value.parse::<PeerId>() {
                Ok(id) => Ok(id),
                Err(_) => Err(reject::custom(BadRequest("invalid peer id".to_string()))),
            }
        })
    }
}

fn with_network_id(
    network_id: NetworkId,
) -> impl Filter<Extract = (NetworkId,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_id.clone())
}

fn with_bech32_hrp(
    bech32_hrp: Bech32Hrp,
) -> impl Filter<Extract = (Bech32Hrp,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || bech32_hrp.clone())
}

fn with_rest_api_config(
    config: RestApiConfig,
) -> impl Filter<Extract = (RestApiConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_protocol_config(
    config: ProtocolConfig,
) -> impl Filter<Extract = (ProtocolConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_tangle<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = (ResourceHandle<MsTangle<B>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tangle.clone())
}

fn with_storage<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = (ResourceHandle<B>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

fn with_message_submitter(
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = (mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || message_submitter.clone())
}

fn with_peer_manager(
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = (ResourceHandle<PeerManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || peer_manager.clone())
}

fn with_network_controller(
    network_controller: ResourceHandle<NetworkController>,
) -> impl Filter<Extract = (ResourceHandle<NetworkController>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_controller.clone())
}
