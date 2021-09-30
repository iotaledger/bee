// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{MessageData, Tangle};

use bee_message::MessageId;

use std::{collections::HashSet, sync::Arc};

///
pub struct TangleWalkerBuilder<'a> {
    tangle: &'a Tangle,
    root: MessageId,
    on_message: Option<Box<dyn Fn(&MessageData) -> bool>>,
    on_missing: Option<Box<dyn Fn(&MessageId)>>,
}

impl<'a> TangleWalkerBuilder<'a> {
    ///
    pub fn new(tangle: &'a Tangle, root: MessageId) -> Self {
        Self {
            tangle,
            root,
            on_message: None,
            on_missing: None,
        }
    }

    pub fn with_on_message(mut self, on_message: Box<dyn Fn(&MessageData) -> bool>) -> Self {
        self.on_message.replace(on_message);
        self
    }

    pub fn with_on_missing(mut self, on_missing: Box<dyn Fn(&MessageId)>) -> Self {
        self.on_missing.replace(on_missing);
        self
    }

    pub fn finish(self) -> TangleWalker<'a> {
        TangleWalker {
            tangle: self.tangle,
            parents: vec![self.root],
            visited: HashSet::new(),
            on_message: self.on_message.unwrap_or_else(|| Box::new(|_| true)),
            on_missing: self.on_missing.unwrap_or_else(|| Box::new(|_| {})),
        }
    }
}

///
pub struct TangleWalker<'a> {
    tangle: &'a Tangle,
    parents: Vec<MessageId>,
    visited: HashSet<MessageId>,
    on_message: Box<dyn Fn(&MessageData) -> bool>,
    on_missing: Box<dyn Fn(&MessageId)>,
}

impl<'a> TangleWalker<'a> {
    ///
    pub fn new(tangle: &'a Tangle, root: MessageId) -> Self {
        TangleWalkerBuilder::new(tangle, root).finish()
    }
}

impl<'a> Iterator for TangleWalker<'a> {
    type Item = (MessageId, Arc<MessageData>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let message_id = self.parents.pop()?;

            if !self.visited.contains(&message_id) {
                self.visited.insert(message_id);

                match self.tangle.get(&message_id) {
                    Some(message_data) => {
                        if (self.on_message)(&message_data) {
                            self.parents
                                .extend(message_data.message().parents().iter().map(|p| p.id()));
                            return Some((message_id, message_data));
                        } else {
                            continue;
                        }
                    }
                    None => {
                        (self.on_missing)(&message_id);
                        continue;
                    }
                }
            }
        }
    }
}
