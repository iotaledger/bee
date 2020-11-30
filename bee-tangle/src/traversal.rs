// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Collection of Tangle traversal functions.

// TODO: Refactor all of this into methods on `Tangle`.

use crate::{
    tangle::{Hooks, Tangle},
    MessageRef,
};

use bee_message::MessageId;

use std::collections::HashSet;

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// the *parent1* edge. The walk continues as long as the visited vertices match a certain condition. For each
/// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
/// associated data and metadata.
pub fn visit_parents_follow_parent1<Metadata, Match, Apply, H: Hooks<Metadata>>(
    tangle: &Tangle<Metadata, H>,
    mut message_id: MessageId,
    mut matches: Match,
    mut apply: Apply,
) where
    Metadata: Clone + Copy,
    Match: FnMut(&MessageRef, &Metadata) -> bool,
    Apply: FnMut(&MessageId, &MessageRef, &Metadata),
{
    while let Some(vtx) = tangle.vertices.get(&message_id) {
        let vtx = vtx.value();

        if !matches(vtx.message(), vtx.metadata()) {
            break;
        } else {
            apply(&message_id, vtx.message(), vtx.metadata());
            message_id = *vtx.parent1();
        }
    }
}

/// A Tangle walker that - given a starting vertex - visits all of its children that are connected through
/// the *parent1* edge. The walk continues as long as the visited vertices match a certain condition. For each
/// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
/// associated data and metadata.
pub fn visit_children_follow_parent1<Metadata, Match, Apply, H: Hooks<Metadata>>(
    tangle: &Tangle<Metadata, H>,
    root: MessageId,
    mut matches: Match,
    mut apply: Apply,
) where
    Metadata: Clone + Copy,
    Match: FnMut(&MessageRef, &Metadata) -> bool,
    Apply: FnMut(&MessageId, &MessageRef, &Metadata),
{
    // TODO could be simplified like visit_parents_follow_parent1 ? Meaning no vector ?
    let mut children = vec![root];

    while let Some(ref parent_id) = children.pop() {
        if let Some(parent) = tangle.vertices.get(parent_id) {
            if matches(parent.value().message(), parent.value().metadata()) {
                apply(parent_id, parent.value().message(), parent.value().metadata());

                if let Some(parent_children) = tangle.children.get(parent_id) {
                    for child_id in parent_children.value() {
                        if let Some(child) = tangle.vertices.get(child_id) {
                            if child.value().parent1() == parent_id {
                                children.push(*child_id);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// either the *parent1* or the *parent2* edge. The walk continues as long as the visited vertices match a certain
/// condition. For each visited vertex customized logic can be applied depending on the availability of the
/// vertex. Each traversed vertex provides read access to its associated data and metadata.
pub fn visit_parents_depth_first<Metadata, Match, Apply, ElseApply, MissingApply, H: Hooks<Metadata>>(
    tangle: &Tangle<Metadata, H>,
    root: MessageId,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
    mut missing_apply: MissingApply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&MessageId, &MessageRef, &Metadata) -> bool,
    Apply: FnMut(&MessageId, &MessageRef, &Metadata),
    ElseApply: FnMut(&MessageId, &MessageRef, &Metadata),
    MissingApply: FnMut(&MessageId),
{
    let mut parents = Vec::new();
    let mut visited = HashSet::new();

    parents.push(root);

    while let Some(message_id) = parents.pop() {
        if !visited.contains(&message_id) {
            match tangle.vertices.get(&message_id) {
                Some(vtx) => {
                    let vtx = vtx.value();

                    if matches(&message_id, vtx.message(), vtx.metadata()) {
                        apply(&message_id, vtx.message(), vtx.metadata());

                        parents.push(*vtx.parent1());
                        parents.push(*vtx.parent2());
                    } else {
                        else_apply(&message_id, vtx.message(), vtx.metadata());
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

// TODO: test
/// A Tangle walker that - given a starting vertex - visits all of its decendents that are connected through
/// either the *parent1* or the *parent2* edge. The walk continues as long as the visited vertices match a certain
/// condition. For each visited vertex customized logic can be applied depending on the availability of the
/// vertex. Each traversed vertex provides read access to its associated data and metadata.
pub fn visit_children_depth_first<Metadata, Match, Apply, ElseApply, H: Hooks<Metadata>>(
    tangle: &Tangle<Metadata, H>,
    root: MessageId,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&MessageRef, &Metadata) -> bool,
    Apply: FnMut(&MessageId, &MessageRef, &Metadata),
    ElseApply: FnMut(&MessageId),
{
    let mut children = vec![root];
    let mut visited = HashSet::new();

    while let Some(message_id) = children.last() {
        match tangle.vertices.get(message_id) {
            Some(r) => {
                let vtx = r.value();

                if visited.contains(vtx.parent1()) && visited.contains(vtx.parent2()) {
                    apply(message_id, vtx.message(), vtx.metadata());
                    visited.insert(*message_id);
                    children.pop();
                } else if !visited.contains(vtx.parent1()) && matches(vtx.message(), vtx.metadata()) {
                    children.push(*vtx.parent1());
                } else if !visited.contains(vtx.parent2()) && matches(vtx.message(), vtx.metadata()) {
                    children.push(*vtx.parent2());
                }
            }
            None => {
                else_apply(message_id);
                visited.insert(*message_id);
                children.pop();
            }
        }
    }
}
