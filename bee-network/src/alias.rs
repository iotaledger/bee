// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// A functional macro that creates a shorter peer alias from the tail of a [`PeerId`] most useful in logging scenarios
/// where the long string representation of ids would make things harder to read.
#[macro_export]
macro_rules! alias {
    ($peer_id:expr) => {
        &$peer_id.to_base58()[46..]
    };
    ($peer_id:expr, $len:expr) => {
        &$peer_id.to_base58()[(52 - $len)..]
    };
}
