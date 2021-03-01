// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{
        RestApiConfig, ROUTE_ADD_PEER, ROUTE_BALANCE_BECH32, ROUTE_BALANCE_ED25519, ROUTE_HEALTH, ROUTE_INFO,
        ROUTE_MESSAGE, ROUTE_MESSAGES_FIND, ROUTE_MESSAGE_CHILDREN, ROUTE_MESSAGE_METADATA, ROUTE_MESSAGE_RAW,
        ROUTE_MILESTONE, ROUTE_MILESTONE_UTXO_CHANGES, ROUTE_OUTPUT, ROUTE_OUTPUTS_BECH32, ROUTE_OUTPUTS_ED25519,
        ROUTE_PEER, ROUTE_PEERS, ROUTE_RECEIPTS, ROUTE_RECEIPTS_AT, ROUTE_REMOVE_PEER, ROUTE_SUBMIT_MESSAGE,
        ROUTE_SUBMIT_MESSAGE_RAW, ROUTE_TIPS, ROUTE_TREASURY,
    },
    handlers, path_params,
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
    Bech32Hrp, NetworkId,
};

use bee_network::NetworkServiceController;
use bee_protocol::{config::ProtocolConfig, MessageSubmitterWorkerEvent, PeerManager};
use bee_runtime::{node::NodeInfo, resource::ResourceHandle};
use bee_tangle::MsTangle;

use tokio::sync::mpsc;
use warp::{reject, Filter, Rejection, Reply};

use std::{collections::HashMap, net::IpAddr};

pub fn all<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    storage: ResourceHandle<B>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    peer_manager: ResourceHandle<PeerManager>,
    network_controller: ResourceHandle<NetworkServiceController>,
    node_info: ResourceHandle<NodeInfo>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    health(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        peer_manager.clone(),
    )
    .or(info(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        network_id.clone(),
        bech32_hrp,
        rest_api_config.clone(),
        protocol_config.clone(),
        node_info,
        peer_manager.clone(),
    )
    .or(tips(public_routes.clone(), allowed_ips.clone(), tangle.clone()))
    .or(submit_message(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        message_submitter.clone(),
        network_id,
        rest_api_config,
        protocol_config,
    ))
    .or(submit_message_raw(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        message_submitter,
    ))
    .or(message_indexation(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(message(public_routes.clone(), allowed_ips.clone(), tangle.clone()))
    .or(message_metadata(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(message_raw(public_routes.clone(), allowed_ips.clone(), tangle.clone()))
    .or(message_children(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(output(public_routes.clone(), allowed_ips.clone(), storage.clone()))
    .or(balance_bech32(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(balance_ed25519(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(outputs_bech32(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(outputs_ed25519(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(milestone(public_routes.clone(), allowed_ips.clone(), tangle))
    .or(milestone_utxo_changes(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(peers(public_routes.clone(), allowed_ips.clone(), peer_manager.clone()))
    .or(peer_add(
        public_routes.clone(),
        allowed_ips.clone(),
        peer_manager.clone(),
        network_controller.clone(),
    ))
    .or(peer_remove(
        public_routes.clone(),
        allowed_ips.clone(),
        network_controller,
    ))
    .or(peer(public_routes.clone(), allowed_ips.clone(), peer_manager))
    .or(receipts(public_routes.clone(), allowed_ips.clone(), storage.clone()))
    .or(receipts_at(public_routes.clone(), allowed_ips.clone(), storage.clone()))
    .or(treasury(public_routes.clone(), allowed_ips.clone(), storage))
    .or(white_flag(public_routes, allowed_ips)))
}

fn health<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("health")
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_HEALTH, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and(with_peer_manager(peer_manager))
        .and_then(handlers::health::health)
}

fn info<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    node_info: ResourceHandle<NodeInfo>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_INFO, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and(with_network_id(network_id))
        .and(with_bech32_hrp(bech32_hrp))
        .and(with_rest_api_config(rest_api_config))
        .and(with_protocol_config(protocol_config))
        .and(with_node_info(node_info))
        .and(with_peer_manager(peer_manager))
        .and_then(handlers::api::v1::info::info)
}

fn tips<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("tips"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_TIPS, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(handlers::api::v1::tips::tips)
}

fn submit_message<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::post())
        .and(has_permission(ROUTE_SUBMIT_MESSAGE, public_routes, allowed_ips))
        .and(warp::body::json())
        .and(with_tangle(tangle))
        .and(with_message_submitter(message_submitter))
        .and(with_network_id(network_id))
        .and(with_rest_api_config(rest_api_config))
        .and(with_protocol_config(protocol_config))
        .and_then(handlers::api::v1::submit_message::submit_message)
}

fn submit_message_raw<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::post())
        .and(has_permission(ROUTE_SUBMIT_MESSAGE_RAW, public_routes, allowed_ips))
        .and(warp::body::bytes())
        .and(with_tangle(tangle))
        .and(with_message_submitter(message_submitter))
        .and_then(handlers::api::v1::submit_message_raw::submit_message_raw)
}

fn message_indexation<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGES_FIND, public_routes, allowed_ips))
        .and(warp::query().and_then(|query: HashMap<String, String>| async move {
            match query.get("index") {
                Some(i) => Ok(i.to_string()),
                None => Err(reject::custom(CustomRejection::BadRequest(
                    "invalid query parameter".to_string(),
                ))),
            }
        }))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::messages_find::messages_find)
}

fn message<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(path_params::message_id())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(handlers::api::v1::message::message)
}

fn message_metadata<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(path_params::message_id())
        .and(warp::path("metadata"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_METADATA, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(handlers::api::v1::message_metadata::message_metadata)
}

fn message_raw<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(path_params::message_id())
        .and(warp::path("raw"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_RAW, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(handlers::api::v1::message_raw::message_raw)
}

fn message_children<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(path_params::message_id())
        .and(warp::path("children"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_CHILDREN, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(handlers::api::v1::message_children::message_children)
}

fn output<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("outputs"))
        .and(path_params::output_id())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUT, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::output::output)
}

fn balance_bech32<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(path_params::bech32_address())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_BALANCE_BECH32, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::balance_bech32::balance_bech32)
}

fn balance_ed25519<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(path_params::ed25519_address())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_BALANCE_ED25519, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::balance_ed25519::balance_ed25519)
}

fn outputs_bech32<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(path_params::bech32_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUTS_BECH32, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::outputs_bech32::outputs_bech32)
}

fn outputs_ed25519<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(path_params::ed25519_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUTS_ED25519, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::outputs_ed25519::outputs_ed25519)
}

fn milestone<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(path_params::milestone_index())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(handlers::api::v1::milestone::milestone)
}

fn milestone_utxo_changes<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("milestones"))
        .and(path_params::milestone_index())
        .and(warp::path("utxo-changes"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_MILESTONE_UTXO_CHANGES, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::milestone_utxo_changes::milestone_utxo_changes)
}

fn peers(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("peers"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_PEERS, public_routes, allowed_ips))
        .and(with_peer_manager(peer_manager))
        .and_then(handlers::api::v1::peers::peers)
}

fn peer(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("peers"))
        .and(path_params::peer_id())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_PEER, public_routes, allowed_ips))
        .and(with_peer_manager(peer_manager))
        .and_then(handlers::api::v1::peer::peer)
}

fn peer_add(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    peer_manager: ResourceHandle<PeerManager>,
    network_controller: ResourceHandle<NetworkServiceController>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("peers"))
        .and(warp::path::end())
        .and(warp::post())
        .and(has_permission(ROUTE_ADD_PEER, public_routes, allowed_ips))
        .and(warp::body::json())
        .and(with_peer_manager(peer_manager))
        .and(with_network_controller(network_controller))
        .and_then(handlers::api::v1::add_peer::add_peer)
}

fn peer_remove(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    network_controller: ResourceHandle<NetworkServiceController>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("peers"))
        .and(path_params::peer_id())
        .and(warp::path::end())
        .and(warp::delete())
        .and(has_permission(ROUTE_REMOVE_PEER, public_routes, allowed_ips))
        .and(with_network_controller(network_controller))
        .and_then(handlers::api::v1::remove_peer::remove_peer)
}

fn receipts<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("receipts"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_RECEIPTS, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::receipt::receipts)
}

fn receipts_at<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("receipts"))
        .and(path_params::milestone_index())
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_RECEIPTS_AT, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::receipt::receipts_at)
}

fn treasury<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("treasury"))
        .and(warp::path::end())
        .and(warp::get())
        .and(has_permission(ROUTE_TREASURY, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and_then(handlers::api::v1::treasury::treasury)
}

fn white_flag(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("debug")
        .and(warp::path("whiteflag"))
        .and(warp::path::end())
        .and(warp::post())
        .and(has_permission(ROUTE_INFO, public_routes, allowed_ips))
        .and(warp::body::json())
        .and_then(handlers::debug::white_flag::white_flag)
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
    network_controller: ResourceHandle<NetworkServiceController>,
) -> impl Filter<Extract = (ResourceHandle<NetworkServiceController>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_controller.clone())
}

fn with_node_info(
    node_info: ResourceHandle<NodeInfo>,
) -> impl Filter<Extract = (ResourceHandle<NodeInfo>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || node_info.clone())
}
