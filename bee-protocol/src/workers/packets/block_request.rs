// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! BlockRequest packet of the protocol.

use std::ops::Range;

use bee_block::BlockId;

use crate::workers::packets::Packet;

const MESSAGE_ID_SIZE: usize = 32;
const CONSTANT_SIZE: usize = MESSAGE_ID_SIZE;

/// A packet to request a block.
#[derive(Clone)]
pub(crate) struct BlockRequestPacket {
    /// Block Id of the requested block.
    pub(crate) block_id: BlockId,
}

impl BlockRequestPacket {
    pub(crate) fn new(block_id: BlockId) -> Self {
        Self { block_id }
    }
}

impl Packet for BlockRequestPacket {
    const ID: u8 = 0x03;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut block_id = [0u8; MESSAGE_ID_SIZE];

        block_id.copy_from_slice(&bytes[0..MESSAGE_ID_SIZE]);

        Self {
            block_id: BlockId::from(block_id),
        }
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn to_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(self.block_id.as_ref())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const MESSAGE_ID: [u8; MESSAGE_ID_SIZE] = [
        160, 3, 36, 228, 202, 18, 56, 37, 229, 28, 240, 65, 225, 238, 64, 55, 244, 83, 155, 232, 31, 255, 208, 9, 126,
        21, 82, 57, 180, 237, 182, 101,
    ];

    #[test]
    fn id() {
        assert_eq!(BlockRequestPacket::ID, 3);
    }

    #[test]
    fn size_range() {
        assert!(!BlockRequestPacket::size_range().contains(&(CONSTANT_SIZE - 1)));
        assert!(BlockRequestPacket::size_range().contains(&CONSTANT_SIZE));
        assert!(!BlockRequestPacket::size_range().contains(&(CONSTANT_SIZE + 1)));
    }

    #[test]
    fn size() {
        let packet = BlockRequestPacket::new(BlockId::from(MESSAGE_ID));

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = BlockRequestPacket::new(BlockId::from(MESSAGE_ID));
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.to_bytes(&mut bytes);
        let packet_to = BlockRequestPacket::from_bytes(&bytes);

        assert!(packet_to.block_id.as_ref().eq(&MESSAGE_ID));
    }
}
