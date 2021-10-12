// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{walkers::TangleWalkerItem, MessageData, StorageBackend, Tangle};

use bee_message::MessageId;

use std::collections::{HashSet, VecDeque};

/// A builder for a [`TangleBfsWalker`].
pub struct TangleBfsWalkerBuilder<'a, S> {
    tangle: &'a Tangle<S>,
    root: MessageId,
    #[allow(clippy::type_complexity)]
    condition: Option<Box<dyn Fn(&'a Tangle<S>, &MessageId, &MessageData) -> bool>>,
}

impl<'a, S: StorageBackend> TangleBfsWalkerBuilder<'a, S> {
    /// Creates a new [`TangleBfsWalkerBuilder`].
    pub fn new(tangle: &'a Tangle<S>, root: MessageId) -> Self {
        Self {
            tangle,
            root,
            condition: None,
        }
    }

    /// Adds a condition to the [`TangleBfsWalkerBuilder`].
    pub fn with_condition(mut self, condition: Box<dyn Fn(&'a Tangle<S>, &MessageId, &MessageData) -> bool>) -> Self {
        self.condition.replace(condition);
        self
    }

    /// Finishes the [`TangleBfsWalkerBuilder`] into a [`TangleBfsWalker`].
    pub fn finish(self) -> TangleBfsWalker<'a, S> {
        TangleBfsWalker {
            tangle: self.tangle,
            parents: vec![self.root].into(),
            visited: HashSet::new(),
            condition: self.condition.unwrap_or_else(|| Box::new(|_, _, _| true)),
        }
    }
}

/// A walker that goes through the tangle in a Breadth First Search manner.
pub struct TangleBfsWalker<'a, S> {
    tangle: &'a Tangle<S>,
    parents: VecDeque<MessageId>,
    visited: HashSet<MessageId>,
    condition: Box<dyn Fn(&'a Tangle<S>, &MessageId, &MessageData) -> bool>,
}

impl<'a, S: StorageBackend> TangleBfsWalker<'a, S> {
    /// Creates a new [`TangleBfsWalker`].
    pub fn new(tangle: &'a Tangle<S>, root: MessageId) -> Self {
        TangleBfsWalkerBuilder::new(tangle, root).finish()
    }
}

impl<'a, S: StorageBackend> Iterator for TangleBfsWalker<'a, S> {
    type Item = TangleWalkerItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let message_id = self.parents.pop_front()?;

            if self.visited.insert(message_id) {
                return if let Some(message_data) = self.tangle.get(&message_id) {
                    if (self.condition)(self.tangle, &message_id, &message_data) {
                        self.parents
                            .extend(message_data.message().parents().iter().map(|p| p.id()));
                        Some(TangleWalkerItem::Matched(message_id, message_data))
                    } else {
                        Some(TangleWalkerItem::Skipped(message_id, message_data))
                    }
                } else {
                    Some(TangleWalkerItem::Missing(message_id))
                };
            }
        }
    }
}
