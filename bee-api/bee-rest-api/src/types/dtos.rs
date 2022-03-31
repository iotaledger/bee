// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Receipt;
use bee_message::payload::dto::ReceiptPayloadDto;
#[cfg(feature = "peer")]
use bee_protocol::types::peer::Peer;
use serde::{Deserialize, Serialize};

/// Describes a peer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerDto {
    pub id: String,
    #[serde(rename = "multiAddresses")]
    pub multi_addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub relation: RelationDto,
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gossip: Option<GossipDto>,
}

#[cfg(feature = "peer")]
impl From<&Peer> for PeerDto {
    fn from(peer: &Peer) -> Self {
        PeerDto {
            id: peer.id().to_string(),
            alias: Some(peer.alias().to_string()),
            multi_addresses: vec![peer.address().to_string()],
            relation: {
                if peer.relation().is_known() {
                    RelationDto::Known
                } else if peer.relation().is_unknown() {
                    RelationDto::Unknown
                } else {
                    RelationDto::Autopeered
                }
            },
            connected: peer.is_connected(),
            gossip: Some(GossipDto {
                heartbeat: HeartbeatDto {
                    solid_milestone_index: *peer.solid_milestone_index(),
                    pruned_milestone_index: *peer.pruned_index(),
                    latest_milestone_index: *peer.latest_milestone_index(),
                    connected_neighbors: peer.connected_peers(),
                    synced_neighbors: peer.synced_peers(),
                },
                metrics: MetricsDto {
                    new_messages: peer.metrics().new_messages(),
                    received_messages: peer.metrics().messages_received(),
                    known_messages: peer.metrics().known_messages(),
                    received_message_requests: peer.metrics().message_requests_received(),
                    received_milestone_requests: peer.metrics().milestone_requests_received(),
                    received_heartbeats: peer.metrics().heartbeats_received(),
                    sent_messages: peer.metrics().messages_sent(),
                    sent_message_requests: peer.metrics().message_requests_sent(),
                    sent_milestone_requests: peer.metrics().milestone_requests_sent(),
                    sent_heartbeats: peer.metrics().heartbeats_sent(),
                    dropped_packets: 0,
                },
            }),
        }
    }
}

/// Returns all information about the gossip stream with the peer.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GossipDto {
    pub heartbeat: HeartbeatDto,
    pub metrics: MetricsDto,
}

/// Describes the relation with the peer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RelationDto {
    #[serde(rename = "known")]
    Known,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "autopeered")]
    Autopeered,
}

/// Describes the heartbeat of a node.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeartbeatDto {
    #[serde(rename = "solidMilestoneIndex")]
    pub solid_milestone_index: u32,
    #[serde(rename = "prunedMilestoneIndex")]
    pub pruned_milestone_index: u32,
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    #[serde(rename = "connectedNeighbors")]
    pub connected_neighbors: u8,
    #[serde(rename = "syncedNeighbors")]
    pub synced_neighbors: u8,
}

/// Describes metrics of a gossip stream.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MetricsDto {
    #[serde(rename = "newMessages")]
    pub new_messages: u64,
    #[serde(rename = "receivedMessages")]
    pub received_messages: u64,
    #[serde(rename = "knownMessages")]
    pub known_messages: u64,
    #[serde(rename = "receivedMessageRequests")]
    pub received_message_requests: u64,
    #[serde(rename = "receivedMilestoneRequests")]
    pub received_milestone_requests: u64,
    #[serde(rename = "receivedHeartbeats")]
    pub received_heartbeats: u64,
    #[serde(rename = "sentMessages")]
    pub sent_messages: u64,
    #[serde(rename = "sentMessageRequests")]
    pub sent_message_requests: u64,
    #[serde(rename = "sentMilestoneRequests")]
    pub sent_milestone_requests: u64,
    #[serde(rename = "sentHeartbeats")]
    pub sent_heartbeats: u64,
    #[serde(rename = "droppedPackets")]
    pub dropped_packets: u64,
}

/// Describes a receipt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptDto {
    pub receipt: ReceiptPayloadDto,
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: u32,
}

impl From<Receipt> for ReceiptDto {
    fn from(value: Receipt) -> Self {
        ReceiptDto {
            receipt: value.inner().into(),
            milestone_index: **value.included_in(),
        }
    }
}

/// Describes the ledger inclusion state of a transaction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LedgerInclusionStateDto {
    #[serde(rename = "conflicting")]
    Conflicting,
    #[serde(rename = "included")]
    Included,
    #[serde(rename = "noTransaction")]
    NoTransaction,
}
