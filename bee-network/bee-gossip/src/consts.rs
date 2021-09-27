// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// The protocol version this node implements.
pub const VERSION: u32 = 0;

/// The maximum packet size transmitted over the wire.
pub const MAX_PACKET_SIZE: usize = 64 * 1024;

/// The maximum handshake packet size allowed.
pub const MAX_HANDSHAKE_PACKET_SIZE: usize = 256;

/// The maximum delay for a handshake response to be accepted.
pub const HANDSHAKE_TIMEOUT_SECS: u64 = 20;

/// TODO
pub const _HANDSHAKE_WIRE_TIMEOUT_MILLIS: u64 = 500;

// From the GoShimmer docs:
// IOTA is a predeclared identifier representing the untyped integer ordinal
// number of the current const specification in a (usually parenthesized)
// const declaration. It is zero-indexed.
pub const IOTA: u8 = 0;
