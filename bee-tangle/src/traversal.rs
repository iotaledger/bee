// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Collection of Tangle traversal functions.

// TODO: Refactor all of this into methods on `Tangle`.

use crate::{metadata::MessageMetadata, storage::StorageBackend, tangle::Tangle, MessageRef};

use bee_message::MessageId;

use std::collections::HashSet;

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// either the *parent1* or the *parent2* edge. The walk continues as long as the visited vertices match a certain
/// condition. For each visited vertex customized logic can be applied depending on the availability of the
/// vertex. Each traversed vertex provides read access to its associated data and metadata.
pub async fn visit_parents_depth_first<Match, Apply, ElseApply, MissingApply, B: StorageBackend>(
    tangle: &Tangle<B>,
    root: MessageId,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
    mut missing_apply: MissingApply,
) where
    Match: Fn(MessageId, MessageRef, MessageMetadata) -> bool,
    Apply: FnMut(&MessageId, &MessageRef, &MessageMetadata),
    ElseApply: FnMut(&MessageId, &MessageRef, &MessageMetadata),
    MissingApply: FnMut(&MessageId),
{
    let mut parents = vec![root];
    let mut visited = HashSet::new();

    while let Some(message_id) = parents.pop() {
        if visited.insert(message_id) {
            let msg_meta = tangle.get_message_and_metadata(&message_id).await;
            match msg_meta {
                Some((msg, meta)) => {
                    if matches(message_id, msg.clone(), meta) {
                        apply(&message_id, &msg, &meta);

                        parents.extend_from_slice(msg.parents());
                    } else {
                        else_apply(&message_id, &msg, &meta);
                    }
                }
                None => {
                    missing_apply(&message_id);
                }
            }
        }
    }
}
