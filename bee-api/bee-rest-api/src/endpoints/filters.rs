// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend, Bech32Hrp, NetworkId};

use bee_gossip::NetworkCommandSender;
use bee_ledger::workers::consensus::ConsensusWorkerCommand;
use bee_protocol::workers::{
    config::ProtocolConfig, MessageRequesterWorker, MessageSubmitterWorkerEvent, PeerManager, RequestedMessages,
};
use bee_runtime::{event::Bus, node::NodeInfo, resource::ResourceHandle};
use bee_tangle::Tangle;

use tokio::sync::mpsc;
use warp::Filter;

use std::convert::Infallible;

pub(crate) fn with_network_id(
    network_id: NetworkId,
) -> impl Filter<Extract = (NetworkId,), Error = Infallible> + Clone {
    warp::any().map(move || network_id.clone())
}

pub(crate) fn with_bech32_hrp(
    bech32_hrp: Bech32Hrp,
) -> impl Filter<Extract = (Bech32Hrp,), Error = Infallible> + Clone {
    warp::any().map(move || bech32_hrp.clone())
}

pub(crate) fn with_rest_api_config(
    config: RestApiConfig,
) -> impl Filter<Extract = (RestApiConfig,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}

pub(crate) fn with_protocol_config(
    config: ProtocolConfig,
) -> impl Filter<Extract = (ProtocolConfig,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
}

pub(crate) fn with_tangle<B: StorageBackend>(
    tangle: ResourceHandle<Tangle<B>>,
) -> impl Filter<Extract = (ResourceHandle<Tangle<B>>,), Error = Infallible> + Clone {
    warp::any().map(move || tangle.clone())
}

pub(crate) fn with_storage<B: StorageBackend>(
    storage: ResourceHandle<B>,
) -> impl Filter<Extract = (ResourceHandle<B>,), Error = Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

pub(crate) fn with_message_submitter(
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = (mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,), Error = Infallible> + Clone {
    warp::any().map(move || message_submitter.clone())
}

pub(crate) fn with_peer_manager(
    peer_manager: ResourceHandle<PeerManager>,
) -> impl Filter<Extract = (ResourceHandle<PeerManager>,), Error = Infallible> + Clone {
    warp::any().map(move || peer_manager.clone())
}

pub(crate) fn with_network_command_sender(
    command_sender: ResourceHandle<NetworkCommandSender>,
) -> impl Filter<Extract = (ResourceHandle<NetworkCommandSender>,), Error = Infallible> + Clone {
    warp::any().map(move || command_sender.clone())
}

pub(crate) fn with_node_info(
    node_info: ResourceHandle<NodeInfo>,
) -> impl Filter<Extract = (ResourceHandle<NodeInfo>,), Error = Infallible> + Clone {
    warp::any().map(move || node_info.clone())
}

pub(crate) fn with_bus(
    bus: ResourceHandle<Bus>,
) -> impl Filter<Extract = (ResourceHandle<Bus>,), Error = Infallible> + Clone {
    warp::any().map(move || bus.clone())
}

pub(crate) fn with_message_requester(
    message_requester: MessageRequesterWorker,
) -> impl Filter<Extract = (MessageRequesterWorker,), Error = Infallible> + Clone {
    warp::any().map(move || message_requester.clone())
}

pub(crate) fn with_requested_messages(
    requested_messages: ResourceHandle<RequestedMessages>,
) -> impl Filter<Extract = (ResourceHandle<RequestedMessages>,), Error = Infallible> + Clone {
    warp::any().map(move || requested_messages.clone())
}

pub(crate) fn with_consensus_worker(
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> impl Filter<Extract = (mpsc::UnboundedSender<ConsensusWorkerCommand>,), Error = Infallible> + Clone {
    warp::any().map(move || consensus_worker.clone())
}
