// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// A macro that creates a shorter representation of a [`PeerId`](crate::PeerId). Mostly useful in logging scenarios.
///
/// **NOTE**: This macro can panic if not used with a valid [`PeerId`](crate::PeerId), or provided with a `len > 52`.
#[macro_export]
macro_rules! alias {
    ($peer_id:expr) => {
        &$peer_id.to_base58()[46..]
    };
    ($peer_id:expr, $len:expr) => {
        &$peer_id.to_base58()[(52 - $len)..]
    };
}
