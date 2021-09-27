// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_identity::identity::PeerId;

use crypto::hashes::sha;
use prost::bytes::{BufMut, BytesMut};

/// Uses the following metric to determine the distance between two peers `a` and `b`:
/// ```
/// xor(hash(a), hash(b+salt))[:4]
/// ```
pub fn peer_distance(a: PeerId, b: PeerId) -> u32 {
    todo!("peer_distance")
}
