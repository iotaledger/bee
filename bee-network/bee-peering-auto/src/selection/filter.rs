// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_identity::PeerId;

use std::collections::HashMap;

type Peer = ();

#[derive(Default)]
pub struct Filter {
    _internal: HashMap<PeerId, bool>,
    _conditions: Vec<Box<dyn Fn(Peer) -> bool>>,
}

impl Filter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Default::default()
    }

    #[allow(dead_code)]
    pub fn add_condition(&mut self, c: Box<dyn Fn(Peer) -> bool>) {
        self._conditions.push(c);
    }
}
