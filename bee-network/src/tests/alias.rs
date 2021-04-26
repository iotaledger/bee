// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::common::keys_and_ids::gen_constant_peer_id;

use crate::alias;

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
