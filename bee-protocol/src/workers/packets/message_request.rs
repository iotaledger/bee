// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! MessageRequest packet of the protocol.

use crate::workers::packets::Packet;

use bee_message::MessageId;

use std::ops::Range;

const MESSAGE_ID_SIZE: usize = 32;
const CONSTANT_SIZE: usize = MESSAGE_ID_SIZE;

/// A packet to request a message.
#[derive(Clone)]
pub(crate) struct MessageRequestPacket {
    /// Message Id of the requested message.
    pub(crate) message_id: MessageId,
}

impl MessageRequestPacket {
    pub(crate) fn new(message_id: MessageId) -> Self {
        Self { message_id }
    }
}

impl Packet for MessageRequestPacket {
    const ID: u8 = 0x03;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut message_id = [0u8; MESSAGE_ID_SIZE];

        message_id.copy_from_slice(&bytes[0..MESSAGE_ID_SIZE]);

        Self {
            message_id: MessageId::from(message_id),
        }
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        bytes.copy_from_slice(self.message_id.as_ref())
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
        assert_eq!(MessageRequestPacket::ID, 3);
    }

    #[test]
    fn size_range() {
        assert_eq!(MessageRequestPacket::size_range().contains(&(CONSTANT_SIZE - 1)), false);
        assert_eq!(MessageRequestPacket::size_range().contains(&CONSTANT_SIZE), true);
        assert_eq!(MessageRequestPacket::size_range().contains(&(CONSTANT_SIZE + 1)), false);
    }

    #[test]
    fn size() {
        let packet = MessageRequestPacket::new(MessageId::from(MESSAGE_ID));

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = MessageRequestPacket::new(MessageId::from(MESSAGE_ID));
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.into_bytes(&mut bytes);
        let packet_to = MessageRequestPacket::from_bytes(&bytes);

        assert!(packet_to.message_id.as_ref().eq(&MESSAGE_ID));
    }
}
