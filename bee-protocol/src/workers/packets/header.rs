// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Header of the type-length-value encoding.

const HEADER_TYPE_SIZE: usize = 1;
const HEADER_LENGTH_SIZE: usize = 2;
pub(crate) const HEADER_SIZE: usize = HEADER_TYPE_SIZE + HEADER_LENGTH_SIZE;

/// A header for the type-length-value encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderPacket {
    /// Type of the packet.
    pub(crate) packet_type: u8,
    /// Length of the packet.
    pub(crate) packet_length: u16,
}

impl HeaderPacket {
    pub(crate) fn from_bytes(bytes: &[u8; HEADER_SIZE]) -> Self {
        // This never panics because `HEADER_TYPE_SIZE < HEADER_SIZE`.
        let (packet_type_bytes, packet_length_bytes) = bytes.split_at(HEADER_TYPE_SIZE);
        Self {
            packet_type: packet_type_bytes[0],
            // This never panics because `packet_length_bytes` has exactly
            // `HEADER_SIZE - HEADER_TYPE_SIZE` bytes by construction.
            packet_length: u16::from_le_bytes(packet_length_bytes.try_into().unwrap()),
        }
    }

    pub(crate) fn to_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = self.packet_type;
        bytes[1..].copy_from_slice(&self.packet_length.to_le_bytes());
    }
}
