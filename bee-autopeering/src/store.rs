// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{identity::PeerId, peer::Peer};

use std::collections::HashMap;

// store peers in order (distance)

pub struct InMemoryPeerStore {
    peers: HashMap<PeerId, Peer>,
}
