// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_identity::identity::PeerId;

/// Uses the following metric to determine the distance between two peers `a` and `b`:
/// ```ignore
/// xor(hash(a), hash(b + salt))[..4]
/// ```
#[allow(dead_code)]
pub fn peer_distance(_a: PeerId, _b: PeerId) -> u32 {
    todo!("peer_distance")
}
