// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Heartbeat packet of the protocol.

use crate::workers::packets::Packet;

use std::ops::Range;

const SOLID_MILESTONE_INDEX_SIZE: usize = 4;
const PRUNED_INDEX_SIZE: usize = 4;
const LATEST_MILESTONE_INDEX_SIZE: usize = 4;
const CONNECTED_PEERS_SIZE: usize = 1;
const SYNCED_PEERS_SIZE: usize = 1;
const CONSTANT_SIZE: usize = SOLID_MILESTONE_INDEX_SIZE
    + PRUNED_INDEX_SIZE
    + LATEST_MILESTONE_INDEX_SIZE
    + CONNECTED_PEERS_SIZE
    + SYNCED_PEERS_SIZE;

/// A packet that informs about the part of the tangle currently being fully stored by a node.
/// This packet is sent when a node:
/// - just got paired to another node;
/// - did a snapshot and pruned away a part of the tangle;
/// - solidified a new milestone;
/// It also helps other nodes to know if they can ask it a specific message.
#[derive(Clone)]
pub(crate) struct HeartbeatPacket {
    /// Index of the latest solid milestone.
    pub(crate) solid_milestone_index: u32,
    /// Pruned index.
    pub(crate) pruned_index: u32,
    /// Index of the latest milestone.
    pub(crate) latest_milestone_index: u32,
    /// Number of connected peers.
    pub(crate) connected_peers: u8,
    /// Number of synced peers.
    pub(crate) synced_peers: u8,
}

impl HeartbeatPacket {
    pub(crate) fn new(
        solid_milestone_index: u32,
        pruned_index: u32,
        latest_milestone_index: u32,
        connected_peers: u8,
        synced_peers: u8,
    ) -> Self {
        Self {
            solid_milestone_index,
            pruned_index,
            latest_milestone_index,
            connected_peers,
            synced_peers,
        }
    }
}

impl Packet for HeartbeatPacket {
    const ID: u8 = 0x04;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let (bytes, next) = bytes.split_at(SOLID_MILESTONE_INDEX_SIZE);
        let solid_milestone_index = u32::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, next) = next.split_at(PRUNED_INDEX_SIZE);
        let pruned_index = u32::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, next) = next.split_at(LATEST_MILESTONE_INDEX_SIZE);
        let latest_milestone_index = u32::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, next) = next.split_at(CONNECTED_PEERS_SIZE);
        let connected_peers = u8::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        let (bytes, _) = next.split_at(SYNCED_PEERS_SIZE);
        let synced_peers = u8::from_le_bytes(bytes.try_into().expect("Invalid buffer size"));

        Self {
            solid_milestone_index,
            pruned_index,
            latest_milestone_index,
            connected_peers,
            synced_peers,
        }
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        let (bytes, next) = bytes.split_at_mut(SOLID_MILESTONE_INDEX_SIZE);
        bytes.copy_from_slice(&self.solid_milestone_index.to_le_bytes());
        let (bytes, next) = next.split_at_mut(PRUNED_INDEX_SIZE);
        bytes.copy_from_slice(&self.pruned_index.to_le_bytes());
        let (bytes, next) = next.split_at_mut(LATEST_MILESTONE_INDEX_SIZE);
        bytes.copy_from_slice(&self.latest_milestone_index.to_le_bytes());
        let (bytes, next) = next.split_at_mut(CONNECTED_PEERS_SIZE);
        bytes.copy_from_slice(&self.connected_peers.to_le_bytes());
        let (bytes, _) = next.split_at_mut(SYNCED_PEERS_SIZE);
        bytes.copy_from_slice(&self.synced_peers.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const SOLID_MILESTONE_INDEX: u32 = 0x0118_1f9b;
    const PRUNED_INDEX: u32 = 0x3dc2_97b4;
    const LATEST_MILESTONE_INDEX: u32 = 0x60be_20c2;
    const CONNECTED_PEERS: u8 = 12;
    const SYNCED_PEERS: u8 = 5;

    #[test]
    fn id() {
        assert_eq!(HeartbeatPacket::ID, 4);
    }

    #[test]
    fn size_range() {
        assert!(!HeartbeatPacket::size_range().contains(&(CONSTANT_SIZE - 1)));
        assert!(HeartbeatPacket::size_range().contains(&CONSTANT_SIZE));
        assert!(!HeartbeatPacket::size_range().contains(&(CONSTANT_SIZE + 1)));
    }

    #[test]
    fn size() {
        let packet = HeartbeatPacket::new(
            SOLID_MILESTONE_INDEX,
            PRUNED_INDEX,
            LATEST_MILESTONE_INDEX,
            CONNECTED_PEERS,
            SYNCED_PEERS,
        );

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = HeartbeatPacket::new(
            SOLID_MILESTONE_INDEX,
            PRUNED_INDEX,
            LATEST_MILESTONE_INDEX,
            CONNECTED_PEERS,
            SYNCED_PEERS,
        );
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.into_bytes(&mut bytes);
        let packet_to = HeartbeatPacket::from_bytes(&bytes);

        assert_eq!(packet_to.solid_milestone_index, SOLID_MILESTONE_INDEX);
        assert_eq!(packet_to.pruned_index, PRUNED_INDEX);
        assert_eq!(packet_to.latest_milestone_index, LATEST_MILESTONE_INDEX);
        assert_eq!(packet_to.connected_peers, CONNECTED_PEERS);
        assert_eq!(packet_to.synced_peers, SYNCED_PEERS);
    }
}
