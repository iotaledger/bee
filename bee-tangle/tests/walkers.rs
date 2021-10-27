// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_message::MessageId;
use bee_tangle::{
    walkers::{TangleBfsWalkerBuilder, TangleDfsWalkerBuilder, TangleWalkerItem},
    MessageData, StorageBackend, Tangle,
};

use std::collections::HashMap;

fn generic_walker_test<T: Iterator<Item = TangleWalkerItem>>(
    walker: T,
    ids: &HashMap<usize, MessageId>,
    expected_matched: Vec<usize>,
    expected_skipped: Vec<usize>,
    expected_missing: Vec<usize>,
) {
    let expected_matched = expected_matched.into_iter().map(|node| ids[&node]).collect::<Vec<_>>();
    let expected_skipped = expected_skipped.into_iter().map(|node| ids[&node]).collect::<Vec<_>>();
    let expected_missing = expected_missing.into_iter().map(|node| ids[&node]).collect::<Vec<_>>();

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    for status in walker {
        match status {
            TangleWalkerItem::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerItem::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerItem::Missing(message_id) => missing.push(message_id),
        }
    }

    assert_eq!(expected_matched, matched);
    assert_eq!(expected_skipped, skipped);
    assert_eq!(expected_missing, missing);
}

#[allow(clippy::type_complexity)]
fn dfs_walker_test<S: StorageBackend>(
    tangle: &Tangle<S>,
    ids: &HashMap<usize, MessageId>,
    root: usize,
    matched: Vec<usize>,
    skipped: Vec<usize>,
    missing: Vec<usize>,
    condition: Option<Box<dyn Fn(&Tangle<S>, &MessageId, &MessageData) -> bool>>,
) {
    let mut builder = TangleDfsWalkerBuilder::new(tangle, ids[&root]);

    if let Some(condition) = condition {
        builder = builder.with_condition(condition);
    }

    generic_walker_test(builder.finish(), ids, matched, skipped, missing);
}

#[allow(clippy::type_complexity)]
fn bfs_walker_test<S: StorageBackend>(
    tangle: &Tangle<S>,
    ids: &HashMap<usize, MessageId>,
    root: usize,
    matched: Vec<usize>,
    skipped: Vec<usize>,
    missing: Vec<usize>,
    condition: Option<Box<dyn Fn(&Tangle<S>, &MessageId, &MessageData) -> bool>>,
) {
    let mut builder = TangleBfsWalkerBuilder::new(tangle, ids[&root]);

    if let Some(condition) = condition {
        builder = builder.with_condition(condition);
    }

    generic_walker_test(builder.finish(), ids, matched, skipped, missing);
}

#[test]
fn binary_tree() {
    let (tangle, ids) = tangle! {
        8  => [0, 1]
        9  => [2, 3]
        10 => [4, 5]
        11 => [6, 7]
        12 => [8, 9]
        13 => [10, 11]
        14 => [12, 13]
    };

    // Without condition

    dfs_walker_test(
        &tangle,
        &ids,
        14,
        vec![14, 12, 8, 9, 13, 10, 11], // matched
        vec![],                         // skipped
        vec![0, 1, 2, 3, 4, 5, 6, 7],   // missing
        None,
    );
    bfs_walker_test(
        &tangle,
        &ids,
        14,
        vec![14, 12, 13, 8, 9, 10, 11], // matched
        vec![],                         // skipped
        vec![0, 1, 2, 3, 4, 5, 6, 7],   // missing
        None,
    );

    // With condition

    fn condition<S: StorageBackend>(_tangle: &Tangle<S>, message_id: &MessageId, _message_data: &MessageData) -> bool {
        u16::from_le_bytes(message_id.as_ref()[0..2].try_into().unwrap()) % 2 == 0
    }

    dfs_walker_test(
        &tangle,
        &ids,
        14,
        vec![14, 12, 8], // matched
        vec![9, 13],     // skipped
        vec![0, 1],      // missing
        Some(Box::new(condition)),
    );
    bfs_walker_test(
        &tangle,
        &ids,
        14,
        vec![14, 12, 8], // matched
        vec![13, 9],     // skipped
        vec![0, 1],      // missing
        Some(Box::new(condition)),
    );
}

#[test]
fn tangle() {
    let (tangle, ids) = tangle! {
        6  => [0, 1, 2]
        7  => [1, 2]
        8  => [2, 3, 4]
        9  => [4, 5]
        10 => [1, 6, 7]
        11 => [3, 7, 8]
        12 => [4, 8, 9]
        13 => [6, 10, 11]
        14 => [8, 11, 12]
        15 => [11, 13, 14]
    };

    // Without condition

    dfs_walker_test(
        &tangle,
        &ids,
        15,
        vec![15, 11, 7, 8, 13, 6, 10, 14, 12, 9], // matched
        vec![],                                   // skipped
        vec![3, 1, 2, 4, 0, 5],                   // missing
        None,
    );
    bfs_walker_test(
        &tangle,
        &ids,
        15,
        vec![15, 11, 13, 14, 7, 8, 6, 10, 12, 9], // matched
        vec![],                                   // skipped
        vec![3, 1, 2, 4, 0, 5],                   // missing
        None,
    );

    // With condition

    fn condition<S: StorageBackend>(_tangle: &Tangle<S>, message_id: &MessageId, _message_data: &MessageData) -> bool {
        u16::from_le_bytes(message_id.as_ref()[0..2].try_into().unwrap()) % 2 == 1
    }

    dfs_walker_test(
        &tangle,
        &ids,
        15,
        vec![15, 11, 7, 13], // matched
        vec![8, 6, 10, 14],  // skipped
        vec![3, 1, 2],       // missing
        Some(Box::new(condition)),
    );
    bfs_walker_test(
        &tangle,
        &ids,
        15,
        vec![15, 11, 13, 7], // matched
        vec![14, 8, 6, 10],  // skipped
        vec![3, 1, 2],       // missing
        Some(Box::new(condition)),
    );
}

#[test]
fn chain() {
    let (tangle, ids) = tangle! {
        8 => [7]
        7 => [6]
        6 => [5]
        5 => [4]
        4 => [3]
        3 => [2]
        2 => [1]
        1 => [0]
    };

    // Without condition

    dfs_walker_test(
        &tangle,
        &ids,
        8,
        vec![8, 7, 6, 5, 4, 3, 2, 1], // matched
        vec![],                       // skipped
        vec![0],                      // missing
        None,
    );
    bfs_walker_test(
        &tangle,
        &ids,
        8,
        vec![8, 7, 6, 5, 4, 3, 2, 1], // matched
        vec![],                       // skipped
        vec![0],                      // missing
        None,
    );

    // With condition

    fn condition<S: StorageBackend>(_tangle: &Tangle<S>, message_id: &MessageId, _message_data: &MessageData) -> bool {
        u16::from_le_bytes(message_id.as_ref()[0..2].try_into().unwrap()) != 4
    }

    dfs_walker_test(
        &tangle,
        &ids,
        8,
        vec![8, 7, 6, 5], // matched
        vec![4],          // skipped
        vec![],           // missing
        Some(Box::new(condition)),
    );
    bfs_walker_test(
        &tangle,
        &ids,
        8,
        vec![8, 7, 6, 5], // matched
        vec![4],          // skipped
        vec![],           // missing
        Some(Box::new(condition)),
    );
}
