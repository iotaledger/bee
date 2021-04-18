// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend, Bech32Hrp, NetworkId};

use bee_network::NetworkServiceController;
use bee_protocol::workers::{
    config::ProtocolConfig, MessageRequesterWorker, MessageSubmitterWorkerEvent, PeerManager, RequestedMessages,
};
use bee_runtime::{event::Bus, node::NodeInfo, resource::ResourceHandle};
use bee_tangle::MsTangle;

use tokio::sync::mpsc;
use warp::Filter;

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

pub(crate) fn with_bus(
    bus: ResourceHandle<Bus>,
) -> impl Filter<Extract = (ResourceHandle<Bus>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || bus.clone())
}

pub(crate) fn with_message_requester(
    message_requester: MessageRequesterWorker,
) -> impl Filter<Extract = (MessageRequesterWorker,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || message_requester.clone())
}

pub(crate) fn with_requested_messages(
    requested_messages: ResourceHandle<RequestedMessages>,
) -> impl Filter<Extract = (ResourceHandle<RequestedMessages>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || requested_messages.clone())
}
