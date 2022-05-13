// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod add_peer;
pub mod block_metadata;
pub mod info;
pub mod message;
pub mod message_children;
pub mod message_raw;
pub mod milestone_by_milestone_id;
pub mod milestone_by_milestone_index;
pub mod output;
pub mod output_metadata;
pub mod peer;
pub mod peers;
pub mod receipts;
pub mod receipts_at;
pub mod remove_peer;
pub mod submit_message;
pub mod tips;
pub mod transaction_included_message;
pub mod treasury;
pub mod utxo_changes_by_milestone_id;
pub mod utxo_changes_by_milestone_index;

use std::net::IpAddr;

use bee_gossip::NetworkCommandSender;
use bee_ledger::workers::consensus::ConsensusWorkerCommand;
use bee_protocol::workers::{config::ProtocolConfig, MessageSubmitterWorkerEvent, PeerManager};
use bee_runtime::{node::NodeInfo, resource::ResourceHandle};
use bee_tangle::Tangle;
use tokio::sync::mpsc;
use warp::{self, Filter, Rejection, Reply};

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend, Bech32Hrp, NetworkId};

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("v2"))
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
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    add_peer::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        peer_manager.clone(),
        network_command_sender.clone(),
    )
    .or(info::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        network_id,
        bech32_hrp,
        rest_api_config.clone(),
        protocol_config.clone(),
        node_info,
        peer_manager.clone(),
    ))
    .or(message::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(message_children::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(block_metadata::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(message_raw::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(milestone_by_milestone_id::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(milestone_by_milestone_index::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(utxo_changes_by_milestone_id::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        storage.clone(),
    ))
    .or(utxo_changes_by_milestone_index::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(output::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
        consensus_worker.clone(),
    ))
    .or(output_metadata::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
        consensus_worker,
    ))
    .or(peer::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        peer_manager.clone(),
    ))
    .or(peers::filter(public_routes.clone(), allowed_ips.clone(), peer_manager))
    .or(receipts::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(receipts_at::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(remove_peer::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        network_command_sender,
    ))
    .or(submit_message::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        message_submitter,
        rest_api_config,
        protocol_config,
    ))
    .or(tips::filter(public_routes.clone(), allowed_ips.clone(), tangle.clone()))
    .or(treasury::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(transaction_included_message::filter(
        public_routes,
        allowed_ips,
        storage,
        tangle,
    ))
}
