// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;
pub mod health;

use std::net::IpAddr;

use bee_gossip::NetworkCommandSender;
use bee_ledger::workers::consensus::ConsensusWorkerCommand;
use bee_protocol::workers::{
    config::ProtocolConfig, BlockRequesterWorker, BlockSubmitterWorkerEvent, PeerManager, RequestedBlocks,
};
use bee_runtime::{event::Bus, node::NodeInfo, resource::ResourceHandle};
use bee_tangle::Tangle;
use tokio::sync::mpsc;
use warp::{self, Filter, Rejection, Reply};

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend, Bech32Hrp, NetworkId};

#[allow(clippy::too_many_arguments)]
pub(crate) fn filter_all<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
    storage: ResourceHandle<B>,
    block_submitter: mpsc::UnboundedSender<BlockSubmitterWorkerEvent>,
    network_id: NetworkId,
    bech32_hrp: Bech32Hrp,
    rest_api_config: RestApiConfig,
    protocol_config: ProtocolConfig,
    peer_manager: ResourceHandle<PeerManager>,
    network_command_sender: ResourceHandle<NetworkCommandSender>,
    node_info: ResourceHandle<NodeInfo>,
    bus: ResourceHandle<Bus<'static>>,
    block_requester: BlockRequesterWorker,
    requested_blocks: ResourceHandle<RequestedBlocks>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    api::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        storage,
        block_submitter,
        network_id,
        bech32_hrp,
        rest_api_config,
        protocol_config,
        peer_manager.clone(),
        network_command_sender,
        node_info,
        bus,
        block_requester,
        requested_blocks,
        consensus_worker,
    )
    .or(health::filter(public_routes, allowed_ips, tangle, peer_manager))
}
