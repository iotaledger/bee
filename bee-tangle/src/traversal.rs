// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Collection of Tangle traversal functions.

// TODO: Refactor all of this into methods on `Tangle`.

use crate::{
    tangle::{Hooks, Tangle},
    MessageRef,
};

use bee_message::MessageId;

use std::{collections::HashSet, future::Future};

// /// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
// /// the first *parent* edge. The walk continues as long as the visited vertices match a certain condition. For each
// /// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
// /// associated data and metadata.
// pub async fn visit_parents_follow_first_parent<Metadata, Match, Apply, H: Hooks<Metadata>>(
//     tangle: &Tangle<Metadata, H>,
//     mut message_id: MessageId,
//     mut matches: Match,
//     mut apply: Apply,
// ) where
//     Metadata: Clone + Copy,
//     Match: FnMut(&MessageRef, &Metadata) -> bool,
//     Apply: FnMut(&MessageId, &MessageRef, &Metadata),
// {
//     while let Some(vtx) = tangle.get_vertex(&message_id).await {
//         if !matches(vtx.message(), vtx.metadata()) {
//             break;
//         } else {
//             apply(&message_id, vtx.message(), vtx.metadata());
//             message_id = vtx.parents()[0];
//         }
//     }
// }

// /// A Tangle walker that - given a starting vertex - visits all of its children that are connected through
// /// the first *parent* edge. The walk continues as long as the visited vertices match a certain condition. For each
// /// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
// /// associated data and metadata.
// pub async fn visit_children_follow_first_parent<Metadata, Match, Apply, H: Hooks<Metadata>>(
//     tangle: &Tangle<Metadata, H>,
//     root: MessageId,
//     mut matches: Match,
//     mut apply: Apply,
// ) where
//     Metadata: Clone + Copy,
//     Match: FnMut(&MessageRef, &Metadata) -> bool,
//     Apply: FnMut(&MessageId, &MessageRef, &Metadata),
// {
//     // TODO could be simplified like visit_parents_follow_parent1 ? Meaning no vector ?
//     let mut children = vec![root];
//
//     while let Some(ref parent_id) = children.pop() {
//         if let Some(parent) = tangle.get_vertex(parent_id).await {
//             if matches(parent.message(), parent.metadata()) {
//                 apply(parent_id, parent.message(), parent.metadata());
//
//                 if let Some(parent_children) = tangle.get_children(parent_id).await {
//                     for child_id in parent_children {
//                         if let Some(child) = tangle.get_vertex(&child_id).await {
//                             if &child.parents()[0] == parent_id {
//                                 children.push(child_id);
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// either the *parent1* or the *parent2* edge. The walk continues as long as the visited vertices match a certain
/// condition. For each visited vertex customized logic can be applied depending on the availability of the
/// vertex. Each traversed vertex provides read access to its associated data and metadata.
pub async fn visit_parents_depth_first<Fut, Metadata, Match, Apply, ElseApply, MissingApply, H: Hooks<Metadata>>(
    tangle: &Tangle<Metadata, H>,
    root: MessageId,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
    mut missing_apply: MissingApply,
) where
    Fut: Future<Output = bool>,
    Metadata: Clone + Copy,
    Match: Fn(MessageId, MessageRef, Metadata) -> Fut,
    Apply: FnMut(&MessageId, &MessageRef, &Metadata),
    ElseApply: FnMut(&MessageId, &MessageRef, &Metadata),
    MissingApply: FnMut(&MessageId),
{
    let mut parents = Vec::new();
    let mut visited = HashSet::new();

    parents.push(root);

    while let Some(message_id) = parents.pop() {
        if !visited.contains(&message_id) {
            let msg_meta = tangle
                .get_vertex(&message_id)
                .await
                .as_ref()
                .and_then(|v| v.message_and_metadata().cloned());
            match msg_meta {
                Some((msg, meta)) => {
                    if matches(message_id, msg.clone(), meta).await {
                        apply(&message_id, &msg, &meta);

                        for parent in msg.parents().iter() {
                            parents.push(*parent);
                        }
                    } else {
                        else_apply(&message_id, &msg, &meta);
                    }
                }
                None => {
                    missing_apply(&message_id);
                }
            }
            visited.insert(message_id);
        }
    }
}

// TODO reimplement with multiple parents
// // TODO: test
// /// A Tangle walker that - given a starting vertex - visits all of its decendents that are connected through
// /// either the *parent1* or the *parent2* edge. The walk continues as long as the visited vertices match a certain
// /// condition. For each visited vertex customized logic can be applied depending on the availability of the
// /// vertex. Each traversed vertex provides read access to its associated data and metadata.
// pub async fn visit_children_depth_first<Metadata, Match, Apply, ElseApply, H: Hooks<Metadata>>(
//     tangle: &Tangle<Metadata, H>,
//     root: MessageId,
//     matches: Match,
//     mut apply: Apply,
//     mut else_apply: ElseApply,
// ) where
//     Metadata: Clone + Copy,
//     Match: Fn(&MessageRef, &Metadata) -> bool,
//     Apply: FnMut(&MessageId, &MessageRef, &Metadata),
//     ElseApply: FnMut(&MessageId),
// {
//     let mut children = vec![root];
//     let mut visited = HashSet::new();
//
//     while let Some(message_id) = children.last() {
//         match tangle.get_vertex(message_id).await {
//             Some(vtx) => {
//                 if visited.contains(vtx.parent1()) && visited.contains(vtx.parent2()) {
//                     apply(message_id, vtx.message(), vtx.metadata());
//                     visited.insert(*message_id);
//                     children.pop();
//                 } else if !visited.contains(vtx.parent1()) && matches(vtx.message(), vtx.metadata()) {
//                     children.push(*vtx.parent1());
//                 } else if !visited.contains(vtx.parent2()) && matches(vtx.message(), vtx.metadata()) {
//                     children.push(*vtx.parent2());
//                 }
//             }
//             None => {
//                 else_apply(message_id);
//                 visited.insert(*message_id);
//                 children.pop();
//             }
//         }
//     }
// }
