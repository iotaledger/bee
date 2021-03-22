// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{MessageRef, VecSet};

use bee_message::{Message, MessageId};

use std::sync::Arc;

#[derive(Clone)]
pub struct Vertex<T>
where
    T: Clone,
{
    message: Option<(MessageRef, T)>,
    children: (VecSet<MessageId>, bool), // Exhaustive flag
}

impl<T> Vertex<T>
where
    T: Clone,
{
    pub fn empty() -> Self {
        Self {
            message: None,
            children: (VecSet::default(), false),
        }
    }

    pub fn new(message: Message, metadata: T) -> Self {
        Self {
            message: Some((MessageRef(Arc::new(message)), metadata)),
            children: (VecSet::default(), false),
        }
    }

    pub fn parents(&self) -> Option<impl Iterator<Item = &MessageId> + '_> {
        Some(self.message()?.parents().iter())
    }

    pub fn message_and_metadata(&self) -> Option<&(MessageRef, T)> {
        self.message.as_ref()
    }

    pub fn message(&self) -> Option<&MessageRef> {
        self.message_and_metadata().map(|(m, _)| m)
    }

    pub fn metadata(&self) -> Option<&T> {
        self.message_and_metadata().map(|(_, m)| m)
    }

    pub fn metadata_mut(&mut self) -> Option<&mut T> {
        self.message.as_mut().map(|(_, m)| m)
    }

    pub fn add_child(&mut self, child: MessageId) {
        self.children.0.insert(child);
    }

    pub fn children(&self) -> &[MessageId] {
        &self.children.0
    }

    pub fn children_exhaustive(&self) -> bool {
        self.children.1
    }

    /// Set the exhaustive flag. This should not be done if the vertex's children are exhaustive.
    pub(crate) fn set_exhaustive(&mut self) {
        self.children.1 = true;
    }

    pub(crate) fn insert_message_and_metadata(&mut self, msg: Message, meta: T) {
        self.message = Some((MessageRef(Arc::new(msg)), meta));
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use bee_test::transaction::create_random_tx;
//
//     #[test]
//     fn create_new_vertex() {
//         let (_, tx) = create_random_tx();
//         let metadata = 0b0000_0001u8;
//
//         let vtx = Vertex::new(tx.clone(), metadata);
//
//         assert_eq!(tx.parent1(), vtx.parent1());
//         assert_eq!(tx.parent2(), vtx.parent2());
//         assert_eq!(tx, **vtx.message());
//         assert_eq!(metadata, *vtx.metadata());
//     }
//
//     #[test]
//     fn update_vertex_meta() {
//         let (_, tx) = create_random_tx();
//
//         let mut vtx = Vertex::new(tx, 0b0000_0001u8);
//         *vtx.metadata_mut() = 0b1111_1110u8;
//
//         assert_eq!(0b1111_1110u8, *vtx.metadata());
//     }
// }
