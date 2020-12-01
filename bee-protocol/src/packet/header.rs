// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Header of the type-length-value encoding.

use std::convert::TryInto;

const HEADER_TYPE_SIZE: usize = 1;
const HEADER_LENGTH_SIZE: usize = 2;
pub(crate) const HEADER_SIZE: usize = HEADER_TYPE_SIZE + HEADER_LENGTH_SIZE;

/// A header for the type-length-value encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Header {
    /// Type of the packet.
    pub(crate) packet_type: u8,
    /// Length of the packet.
    pub(crate) packet_length: u16,
}

impl Header {
    // TODO impl try_from
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            packet_type: bytes[0],
            // TODO propagate error
            packet_length: u16::from_le_bytes(bytes[HEADER_TYPE_SIZE..HEADER_SIZE].try_into().unwrap()),
        }
    }

    pub(crate) fn to_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = self.packet_type;
        bytes[1..].copy_from_slice(&self.packet_length.to_le_bytes());
    }
}
