// Copyright 2020 IOTA Stiftung
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

#[cfg(test)]
mod tests {
    use crate::util::gen_constant_peer_id;

    #[test]
    fn alias_default() {
        let peer_id = gen_constant_peer_id();
        let alias = alias!(peer_id);
        assert_eq!(alias, "eF27st");
    }

    #[test]
    fn alias_custom() {
        let peer_id = gen_constant_peer_id();
        let alias = alias!(peer_id, 10);
        assert_eq!(alias, "WSUEeF27st");
    }
}
