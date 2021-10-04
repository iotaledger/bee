// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{MessageData, Tangle};

use bee_message::MessageId;

use std::collections::HashSet;

///
#[derive(Debug)]
pub enum TangleWalkerStatus {
    ///
    Matched(MessageId, MessageData),
    ///
    Skipped(MessageId, MessageData),
    ///
    Missing(MessageId),
}

///
pub struct TangleDfsWalkerBuilder<'a> {
    tangle: &'a Tangle,
    root: MessageId,
    condition: Option<Box<dyn Fn(&MessageData) -> bool>>,
}

impl<'a> TangleDfsWalkerBuilder<'a> {
    ///
    pub fn new(tangle: &'a Tangle, root: MessageId) -> Self {
        Self {
            tangle,
            root,
            condition: None,
        }
    }

    ///
    pub fn with_condition(mut self, condition: Box<dyn Fn(&MessageData) -> bool>) -> Self {
        self.condition.replace(condition);
        self
    }

    ///
    pub fn finish(self) -> TangleDfsWalker<'a> {
        TangleDfsWalker {
            tangle: self.tangle,
            parents: vec![self.root],
            visited: HashSet::new(),
            condition: self.condition.unwrap_or_else(|| Box::new(|_| true)),
        }
    }
}

///
pub struct TangleDfsWalker<'a> {
    tangle: &'a Tangle,
    parents: Vec<MessageId>,
    visited: HashSet<MessageId>,
    condition: Box<dyn Fn(&MessageData) -> bool>,
}

impl<'a> TangleDfsWalker<'a> {
    ///
    pub fn new(tangle: &'a Tangle, root: MessageId) -> Self {
        TangleDfsWalkerBuilder::new(tangle, root).finish()
    }
}

impl<'a> Iterator for TangleDfsWalker<'a> {
    type Item = TangleWalkerStatus;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let message_id = self.parents.pop()?;

            if !self.visited.contains(&message_id) {
                self.visited.insert(message_id);

                return match self.tangle.get(&message_id) {
                    Some(message_data) => {
                        if (self.condition)(&message_data) {
                            self.parents
                                .extend(message_data.message().parents().iter().map(|p| p.id()));
                            Some(TangleWalkerStatus::Matched(message_id, message_data))
                        } else {
                            Some(TangleWalkerStatus::Skipped(message_id, message_data))
                        }
                    }
                    None => Some(TangleWalkerStatus::Missing(message_id)),
                };
            }
        }
    }
}
