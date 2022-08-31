// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Collection of Tangle traversal functions.

// TODO: Refactor all of this into methods on `Tangle`.

use std::collections::HashSet;

use bee_block::{Block, BlockId};

use crate::{block_metadata::BlockMetadata, storage::StorageBackend, tangle::Tangle};

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// either the *parent1* or the *parent2* edge. The walk continues as long as the visited vertices match a certain
/// condition. For each visited vertex customized logic can be applied depending on the availability of the
/// vertex. Each traversed vertex provides read access to its associated data and metadata.
pub fn visit_parents_depth_first<Match, Apply, ElseApply, MissingApply, B: StorageBackend>(
    tangle: &Tangle<B>,
    root: BlockId,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
    mut missing_apply: MissingApply,
) where
    Match: Fn(&BlockId, &Block, &BlockMetadata) -> bool,
    Apply: FnMut(&BlockId, &Block, &BlockMetadata),
    ElseApply: FnMut(&BlockId, &Block, &BlockMetadata),
    MissingApply: FnMut(&BlockId),
{
    let mut parents = vec![root];
    let mut visited = HashSet::new();

    while let Some(block_id) = parents.pop() {
        if visited.insert(block_id) {
            let block_and_metadata = tangle.get_block_and_metadata(&block_id);
            match block_and_metadata {
                Some((block, metadata)) => {
                    if matches(&block_id, &block, &metadata) {
                        apply(&block_id, &block, &metadata);

                        parents.extend_from_slice(block.parents());
                    } else {
                        else_apply(&block_id, &block, &metadata);
                    }
                }
                None => {
                    missing_apply(&block_id);
                }
            }
        }
    }
}
