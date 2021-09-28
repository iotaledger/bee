// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_identity::PeerId;

use std::collections::HashMap;

struct Peer;

#[derive(Default)]
pub struct Filter {
    internal: HashMap<PeerId, bool>,
    conditions: Vec<Box<dyn Fn(Peer) -> bool>>,
}

impl Filter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_condition(&mut self, c: Box<dyn Fn(Peer) -> bool>) {
        self.conditions.push(c);
    }
}
