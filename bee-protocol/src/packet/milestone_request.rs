// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! MilestoneRequest packet of the protocol.

use crate::packet::Packet;

use std::{convert::TryInto, ops::Range};

const INDEX_SIZE: usize = 4;
const CONSTANT_SIZE: usize = INDEX_SIZE;

/// A packet to request a milestone.
#[derive(Default)]
pub(crate) struct MilestoneRequest {
    /// Index of the requested milestone.
    pub(crate) index: u32,
}

impl MilestoneRequest {
    pub(crate) fn new(index: u32) -> Self {
        Self { index }
    }
}

impl Packet for MilestoneRequest {
    const ID: u8 = 0x01;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut packet = Self::default();

        packet.index = u32::from_le_bytes(bytes[0..INDEX_SIZE].try_into().expect("Invalid buffer size"));

        packet
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
        assert_eq!(MilestoneRequest::ID, 1);
    }

    #[test]
    fn size_range() {
        assert_eq!(MilestoneRequest::size_range().contains(&3), false);
        assert_eq!(MilestoneRequest::size_range().contains(&4), true);
        assert_eq!(MilestoneRequest::size_range().contains(&5), false);
    }

    #[test]
    fn size() {
        let packet = MilestoneRequest::new(INDEX);

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = MilestoneRequest::new(INDEX);
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.into_bytes(&mut bytes);
        let packet_to = MilestoneRequest::from_bytes(&bytes);

        assert_eq!(packet_to.index, INDEX);
    }
}
