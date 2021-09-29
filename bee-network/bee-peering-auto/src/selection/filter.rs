// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_identity::PeerId;

// FIXME: 1:1 Go port

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type Peer = ();

#[derive(Default, Clone)]
pub struct Filter {
    internal: Arc<RwLock<HashMap<PeerId, bool>>>,
    conditions: Arc<RwLock<Vec<Box<dyn Fn(&Peer) -> bool>>>>,
}

impl Filter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_condition(&self, c: Box<dyn Fn(&Peer) -> bool>) {
        self.conditions.write().expect("lock").push(c);
    }
}
