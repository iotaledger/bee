// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! MessageRequest packet of the protocol.

use crate::packet::Packet;

use std::ops::Range;

const MESSAGE_ID_SIZE: usize = 32;
const CONSTANT_SIZE: usize = MESSAGE_ID_SIZE;

/// A packet to request a message.
pub(crate) struct MessageRequest {
    /// Message Id of the requested message.
    pub(crate) message_id: [u8; MESSAGE_ID_SIZE],
}

impl MessageRequest {
    pub(crate) fn new(message_id: &[u8]) -> Self {
        let mut new = Self::default();

        new.message_id.copy_from_slice(message_id);

        new
    }
}

impl Default for MessageRequest {
    fn default() -> Self {
        Self {
            message_id: [0; MESSAGE_ID_SIZE],
        }
    }
}

impl Packet for MessageRequest {
    const ID: u8 = 0x03;

    fn size_range() -> Range<usize> {
        (CONSTANT_SIZE)..(CONSTANT_SIZE + 1)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut packet = Self::default();

        packet.message_id.copy_from_slice(&bytes[0..MESSAGE_ID_SIZE]);

        packet
    }

    fn size(&self) -> usize {
        CONSTANT_SIZE
    }

    fn into_bytes(self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.message_id)
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
        assert_eq!(MessageRequest::ID, 3);
    }

    #[test]
    fn size_range() {
        assert_eq!(MessageRequest::size_range().contains(&31), false);
        assert_eq!(MessageRequest::size_range().contains(&32), true);
        assert_eq!(MessageRequest::size_range().contains(&33), false);
    }

    #[test]
    fn size() {
        let packet = MessageRequest::new(&MESSAGE_ID);

        assert_eq!(packet.size(), CONSTANT_SIZE);
    }

    #[test]
    fn into_from() {
        let packet_from = MessageRequest::new(&MESSAGE_ID);
        let mut bytes = vec![0u8; packet_from.size()];
        packet_from.into_bytes(&mut bytes);
        let packet_to = MessageRequest::from_bytes(&bytes);

        assert!(packet_to.message_id.eq(&MESSAGE_ID));
    }
}
