// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Utilities to establish and maintain network connections with peers.

#![deny(missing_docs)]

mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

pub mod packet;

/// The current protocol version.
pub const VERSION: u32 = 0;

/// The maximum packet size transmitted over the wire.
pub const MAX_PACKET_SIZE: usize = 64 * 1024;

/// From the GoShimmer docs:
/// IOTA is a predeclared identifier representing the untyped integer ordinal
/// number of the current const specification in a (usually parenthesized)
/// const declaration. It is zero-indexed.
pub const IOTA: u8 = 0;
