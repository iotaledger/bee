// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// A macro that creates a shorter representation of a [`PeerId`](bee_identity::PeerId). Mostly useful in logging scenarios.
///
/// **NOTE**: This macro can panic if not used with a valid [`PeerId`](bee_identity::PeerId).
#[macro_export]
macro_rules! alias {
    ($peer_id:expr) => {
        &$peer_id.to_base58()[8..24]
    };
    ($peer_id:expr, $len:expr) => {
        &$peer_id.to_base58()[8..8 + $len]
    };
}
