// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::RestApiConfig,
    endpoints, 
    storage::StorageBackend,
    Bech32Hrp, NetworkId,
};

use bee_network::NetworkServiceController;
use bee_protocol::{config::ProtocolConfig, MessageSubmitterWorkerEvent, PeerManager};
use bee_runtime::{node::NodeInfo, resource::ResourceHandle};
use bee_tangle::MsTangle;

use tokio::sync::mpsc;
use warp::{Filter, Rejection, Reply};

use std::net::IpAddr;

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
    endpoints::health::health_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        peer_manager.clone(),
    )
    .or(endpoints::api::v1::info::info_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        network_id.clone(),
        bech32_hrp,
        rest_api_config.clone(),
        protocol_config.clone(),
        node_info,
        peer_manager.clone(),
    ))
    .or(endpoints::api::v1::tips::tips_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        tangle.clone(),
    ))
    .or(endpoints::api::v1::submit_message::submit_message_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        message_submitter.clone(),
        network_id,
        rest_api_config,
        protocol_config,
    ))
    .or(endpoints::api::v1::submit_message_raw::submit_message_raw_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        message_submitter,
    ))
    .or(endpoints::api::v1::messages_find::messages_find_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(endpoints::api::v1::message::message_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        tangle.clone(),
    ))
    .or(endpoints::api::v1::message_metadata::message_metadata_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(endpoints::api::v1::message_raw::message_raw_filter(
        public_routes.clone(), 
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(endpoints::api::v1::message_children::message_children_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(endpoints::api::v1::output::output_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        storage.clone()
    ))
    .or(endpoints::api::v1::balance_bech32::balance_bech32_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(endpoints::api::v1::balance_ed25519::balance_ed25519_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(endpoints::api::v1::outputs_bech32::outputs_bech32_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(endpoints::api::v1::outputs_ed25519::outputs_ed25519_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(endpoints::api::v1::milestone::milestone_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        tangle
    ))
    .or(endpoints::api::v1::milestone_utxo_changes::milestone_utxo_changes_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(endpoints::api::v1::peers::peers_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        peer_manager.clone()
    ))
    .or(endpoints::api::v1::add_peer::add_peer_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        peer_manager.clone(),
        network_controller.clone(),
    ))
    .or(endpoints::api::v1::remove_peer::remove_peer_filter(
        public_routes.clone(),
        allowed_ips.clone(),
        network_controller,
    ))
    .or(endpoints::api::v1::peer::peer_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        peer_manager
    ))
    .or(endpoints::api::v1::receipt::receipts_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        storage.clone()
    ))
    .or(endpoints::api::v1::receipt::receipts_at_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        storage.clone()
    ))
    .or(endpoints::api::v1::treasury::treasury_filter(
        public_routes.clone(), 
        allowed_ips.clone(), 
        storage
    ))
    .or(endpoints::debug::white_flag::white_flag_filter(
        public_routes, 
        allowed_ips
    ))
}

pub(crate) fn with_network_id(
    network_id: NetworkId,
) -> impl Filter<Extract = (NetworkId,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_id.clone())
}

pub(crate) fn with_bech32_hrp(
    bech32_hrp: Bech32Hrp,
) -> impl Filter<Extract = (Bech32Hrp,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || bech32_hrp.clone())
}

pub(crate) fn with_rest_api_config(
    config: RestApiConfig,
) -> impl Filter<Extract = (RestApiConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

pub(crate) fn with_protocol_config(
    config: ProtocolConfig,
) -> impl Filter<Extract = (ProtocolConfig,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

pub(crate) fn with_tangle<B: StorageBackend>(
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = (ResourceHandle<MsTangle<B>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tangle.clone())
}

pub(crate) fn with_storage<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = (ResourceHandle<B>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

pub(crate) fn with_message_submitter(
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = (mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || message_submitter.clone())
}

pub(crate) fn with_peer_manager(
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = (ResourceHandle<PeerManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || peer_manager.clone())
}

pub(crate) fn with_network_controller(
    network_controller: ResourceHandle<NetworkServiceController>,
) -> impl Filter<Extract = (ResourceHandle<NetworkServiceController>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_controller.clone())
}

pub(crate) fn with_node_info(
    node_info: ResourceHandle<NodeInfo>,
) -> impl Filter<Extract = (ResourceHandle<NodeInfo>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || node_info.clone())
}
