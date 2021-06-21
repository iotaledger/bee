// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! MilestoneRequest packet of the protocol.

use crate::workers::packets::Packet;

use std::{convert::TryInto, ops::Range};

const INDEX_SIZE: usize = 4;
const CONSTANT_SIZE: usize = INDEX_SIZE;

/// A packet to request a milestone.
#[derive(Clone)]
pub(crate) struct MilestoneRequestPacket {
    /// Index of the requested milestone.
    pub(crate) index: u32,
}

impl MilestoneRequestPacket {
    pub(crate) fn new(index: u32) -> Self {
        Self { index }
    }
}

impl Packet for MilestoneRequestPacket {
    const ID: u8 = 0x01;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            index: u32::from_le_bytes(bytes[0..INDEX_SIZE].try_into().expect("Invalid buffer size")),
        }
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.index.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const INDEX: u32 = 0x81f7_df7c;

    #[test]
    fn id() {
        assert_eq!(MilestoneRequestPacket::ID, 1);
    }

    #[test]
    fn size_range() {
        assert!(!MilestoneRequestPacket::size_range().contains(&(CONSTANT_SIZE - 1)),);
        assert!(MilestoneRequestPacket::size_range().contains(&CONSTANT_SIZE));
        assert!(!MilestoneRequestPacket::size_range().contains(&(CONSTANT_SIZE + 1)),);
    }

    #[test]
    fn size() {
        let packet = MilestoneRequestPacket::new(INDEX);

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = MilestoneRequestPacket::new(INDEX);
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.into_bytes(&mut bytes);
        let packet_to = MilestoneRequestPacket::from_bytes(&bytes);

        assert_eq!(packet_to.index, INDEX);
    }
}
