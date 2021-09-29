// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{MessageData, Tangle};

use bee_message::MessageId;

use std::collections::HashSet;

pub struct TangleWalker<'a> {
    tangle: &'a Tangle,
    parents: Vec<MessageId>,
    visited: HashSet<MessageId>,
    on_message: Box<dyn Fn(&MessageData) -> bool>,
    on_missing: Box<dyn Fn(&MessageId)>,
}

impl<'a> TangleWalker<'a> {
    pub fn new(tangle: &'a Tangle, root: MessageId) -> Self {
        Self {
            tangle,
            parents: vec![root],
            visited: HashSet::new(),
            on_message: Box::new(|_| true),
            on_missing: Box::new(|_| {}),
        }
    }
}

impl<'a> Iterator for TangleWalker<'a> {
    type Item = MessageData;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let message_id = self.parents.pop()?;

            if !self.visited.contains(&message_id) {
                match self.tangle.get(&message_id) {
                    Some(ref message_data) => {
                        if (self.on_message)(message_data) {
                            self.parents
                                .extend(message_data.message().parents().iter().map(|p| p.id()));
                        } else {
                            continue;
                        }
                    }
                    None => {
                        (self.on_missing)(&message_id);
                        continue;
                    }
                }

                self.visited.insert(message_id);
            }
        }
    }
}
