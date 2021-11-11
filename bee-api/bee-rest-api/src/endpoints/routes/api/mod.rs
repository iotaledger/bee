// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod plugins;
pub mod v1;

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend, Bech32Hrp, NetworkId};

use bee_ledger::workers::consensus::ConsensusWorkerCommand;
use bee_gossip::NetworkCommandSender;
use bee_protocol::workers::{
    config::ProtocolConfig, MessageRequesterWorker, MessageSubmitterWorkerEvent, PeerManager, RequestedMessages,
};
use bee_runtime::{event::Bus, node::NodeInfo, resource::ResourceHandle};
use bee_tangle::Tangle;

use tokio::sync::mpsc;
use warp::{self, Filter, Rejection, Reply};

use std::net::IpAddr;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::path("api")
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
    storage: ResourceHandle<B>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    peer_manager: ResourceHandle<PeerManager>,
    network_command_sender: ResourceHandle<NetworkCommandSender>,
    node_info: ResourceHandle<NodeInfo>,
    bus: ResourceHandle<Bus<'static>>,
    message_requester: MessageRequesterWorker,
    requested_messages: ResourceHandle<RequestedMessages>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    v1::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        storage.clone(),
        message_submitter,
        network_id,
        bech32_hrp,
        rest_api_config.clone(),
        protocol_config,
        peer_manager,
        network_command_sender,
        node_info,
        consensus_worker,
    )
    .or(plugins::filter(
        public_routes,
        allowed_ips,
        storage,
        tangle,
        bus,
        message_requester,
        requested_messages,
        rest_api_config,
    ))
}
