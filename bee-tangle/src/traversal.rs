// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Collection of Tangle traversal functions.

// TODO: Refactor all of this into methods on `Tangle`.

use crate::{
    tangle::{Hooks, Tangle},
    MessageRef,
};

use bee_message::MessageId;

use std::{collections::HashSet, future::Future};

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

                        parents.extend_from_slice(&msg.parents());
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
