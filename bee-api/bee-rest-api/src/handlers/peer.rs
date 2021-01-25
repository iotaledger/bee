// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    types::{GossipMetricsDto, PeerDto},
};

use bee_network::PeerId;
use bee_protocol::PeerManager;
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

use crate::filters::CustomRejection::NotFound;

pub(crate) async fn peer(peer_id: PeerId, peer_manager: ResourceHandle<PeerManager>) -> Result<impl Reply, Rejection> {
    match peer_manager.get(&peer_id).await {
        Some(peer_entry) => {
            let peer = &peer_entry.0;
            let peer_dto = PeerDto {
                id: peer.id().to_string(),
                alias: Some(peer.alias().to_string()),
                multi_addresses: vec![peer.address().to_string()],
                relation: {
                    if peer.relation().is_known() {
                        "known".to_string()
                    } else if peer.relation().is_unknown() {
                        "unknown".to_string()
                    } else {
                        "discovered".to_string()
                    }
                },
                connected: peer_manager.is_connected(peer.id()).await,
                gossip_metrics: GossipMetricsDto {
                    received_messages: peer.metrics().messages_received(),
                    new_messages: peer.metrics().new_messages(),
                    known_messages: peer.metrics().known_messages(),
                    received_message_requests: peer.metrics().message_requests_received(),
                    received_milestone_requests: peer.metrics().milestone_requests_received(),
                    received_heartbeats: peer.metrics().heartbeats_received(),
                    sent_messages: peer.metrics().messages_sent(),
                    sent_message_requests: peer.metrics().message_requests_sent(),
                    sent_milestone_requests: peer.metrics().milestone_requests_sent(),
                    sent_heartbeats: peer.metrics().heartbeats_sent(),
                    dropped_packets: peer.metrics().invalid_packets(), /* TODO dropped_packets ==
                                                                        * invalid_packets? */
                },
            };

            Ok(warp::reply::json(&SuccessBody::new(PeerResponse(peer_dto))))
        }

        None => Err(reject::custom(NotFound("peer not found".to_string()))),
    }
}

/// Response of GET /api/v1/peer/{peer_id}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerResponse(pub PeerDto);

impl BodyInner for PeerResponse {}
