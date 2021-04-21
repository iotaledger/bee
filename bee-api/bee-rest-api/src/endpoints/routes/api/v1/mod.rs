// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod add_peer;
pub mod balance_bech32;
pub mod balance_ed25519;
pub mod info;
pub mod message;
pub mod message_children;
pub mod message_metadata;
pub mod message_raw;
pub mod messages_find;
pub mod milestone;
pub mod milestone_utxo_changes;
pub mod output;
pub mod outputs_bech32;
pub mod outputs_ed25519;
pub mod peer;
pub mod peers;
pub mod receipts;
pub mod receipts_at;
pub mod remove_peer;
pub mod submit_message;
pub mod submit_message_raw;
pub mod tips;
pub mod transaction_included_message;
pub mod treasury;

use crate::endpoints::{config::RestApiConfig, storage::StorageBackend, Bech32Hrp, NetworkId};

use bee_network::NetworkServiceController;
use bee_protocol::workers::{config::ProtocolConfig, MessageSubmitterWorkerEvent, PeerManager};
use bee_runtime::{node::NodeInfo, resource::ResourceHandle};
use bee_tangle::MsTangle;

use warp::{self, Filter, Rejection, Reply};

use std::net::IpAddr;

use tokio::sync::mpsc;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("v1"))
}

pub(crate) fn filter<B: StorageBackend>(
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
    add_peer::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        peer_manager.clone(),
        network_controller.clone(),
    )
    .or(balance_bech32::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(balance_ed25519::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(info::filter(
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
    .or(message_metadata::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(message_raw::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(messages_find::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(milestone::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
    ))
    .or(milestone_utxo_changes::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(output::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(outputs_bech32::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
    ))
    .or(outputs_ed25519::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        storage.clone(),
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
        network_controller,
    ))
    .or(submit_message::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        message_submitter.clone(),
        network_id,
        rest_api_config,
        protocol_config,
    ))
    .or(submit_message_raw::filter(
        public_routes.clone(),
        allowed_ips.clone(),
        tangle.clone(),
        message_submitter,
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
