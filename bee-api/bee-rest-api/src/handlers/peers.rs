// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    types::{GossipMetricsDto, PeerDto},
};

use bee_protocol::{Peer, PeerManager};
use bee_runtime::resource::ResourceHandle;

use serde::{Deserialize, Serialize};
use warp::Reply;

use std::{
    convert::Infallible,
    sync::{atomic::Ordering, Arc},
};

pub(crate) async fn peers(peer_manager: ResourceHandle<PeerManager>) -> Result<impl Reply, Infallible> {
    let mut peer_dtos = Vec::new();
    for peer in peer_manager.get_all().await {
        let peer: Arc<Peer> = peer;
        let peer_dto = PeerDto {
            id: peer.id().to_string(),
            alias: peer.alias().to_string(),
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
                received_messages: peer.metrics().messages_received.load(Ordering::Relaxed),
                new_messages: peer.metrics().new_messages.load(Ordering::Relaxed),
                known_messages: peer.metrics().known_messages.load(Ordering::Relaxed),
                received_message_requests: peer.metrics().message_requests_received.load(Ordering::Relaxed),
                received_milestone_requests: peer.metrics().milestone_requests_received.load(Ordering::Relaxed),
                received_heartbeats: peer.metrics().heartbeats_received.load(Ordering::Relaxed),
                sent_messages: peer.metrics().messages_sent.load(Ordering::Relaxed),
                sent_message_requests: peer.metrics().message_requests_sent.load(Ordering::Relaxed),
                sent_milestone_requests: peer.metrics().milestone_requests_sent.load(Ordering::Relaxed),
                sent_heartbeats: peer.metrics().heartbeats_sent.load(Ordering::Relaxed),
                dropped_packets: peer.metrics().invalid_packets.load(Ordering::Relaxed), /* TODO dropped_packets ==
                                                                                          * invalid_packets? */
            },
        };
        peer_dtos.push(peer_dto);
    }

    Ok(warp::reply::json(&SuccessBody::new(PeersResponse(peer_dtos))))
}

/// Response of GET /api/v1/info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeersResponse(pub Vec<PeerDto>);

impl BodyInner for PeersResponse {}
