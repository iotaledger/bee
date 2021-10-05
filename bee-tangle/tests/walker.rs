// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_message::MessageId;
use bee_tangle::walkers::{TangleBfsWalker, TangleDfsWalker, TangleWalkerItem};

#[test]
fn binary_tree_no_condition() {
    // 0 --
    //     | --  8 --
    // 1 --          |
    //               | -- 12 --
    // 2 --          |         |
    //     | --  9 --          |
    // 3 --                    |
    //                         | -- 14
    // 4 --                    |
    //     | -- 10 --          |
    // 5 --          |         |
    //               | -- 13 --
    // 6 --          |
    //     | -- 11 --
    // 7 --

    let (tangle, ids) = tangle! {
        8 => [0, 1]
        9 => [2, 3]
        10 => [4, 5]
        11 => [6, 7]
        12 => [8, 9]
        13 => [10, 11]
        14 => [12, 13]
    };

    let walker = TangleDfsWalker::new(&tangle, ids[&14]);

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    let dfs_matched: Vec<MessageId> = IntoIterator::into_iter([14, 12, 8, 9, 13, 10, 11])
        .map(|node| ids[&node])
        .collect();
    let dfs_missing: Vec<MessageId> = IntoIterator::into_iter([0, 1, 2, 3, 4, 5, 6, 7])
        .map(|node| ids[&node])
        .collect();

    for status in walker {
        match status {
            TangleWalkerItem::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerItem::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerItem::Missing(message_id) => missing.push(message_id),
        }
    }

    assert_eq!(dfs_matched, matched);
    assert!(skipped.is_empty());
    assert_eq!(dfs_missing, missing);

    let walker = TangleBfsWalker::new(&tangle, ids[&14]);

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    let bfs_matched: Vec<MessageId> = IntoIterator::into_iter([14, 12, 13, 8, 9, 10, 11])
        .map(|node| ids[&node])
        .collect();
    let bfs_missing: Vec<MessageId> = IntoIterator::into_iter([0, 1, 2, 3, 4, 5, 6, 7])
        .map(|node| ids[&node])
        .collect();

    for status in walker {
        match status {
            TangleWalkerItem::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerItem::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerItem::Missing(message_id) => missing.push(message_id),
        }
    }

    assert_eq!(bfs_matched, matched);
    assert!(skipped.is_empty());
    assert_eq!(bfs_missing, missing);
}

#[test]
fn chain_no_condition() {
    // 0 - 1 - 2 - 3 - 4 - 5 - 6 - 7 - 8

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

    let walker = TangleDfsWalker::new(&tangle, ids[&8]);

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    let dfs_matched: Vec<MessageId> = IntoIterator::into_iter([8, 7, 6, 5, 4, 3, 2, 1])
        .map(|node| ids[&node])
        .collect();
    let dfs_missing: Vec<MessageId> = IntoIterator::into_iter([0]).map(|node| ids[&node]).collect();

    for status in walker {
        match status {
            TangleWalkerItem::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerItem::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerItem::Missing(message_id) => missing.push(message_id),
        }
    }

    assert_eq!(dfs_matched, matched);
    assert!(skipped.is_empty());
    assert_eq!(dfs_missing, missing);

    let walker = TangleBfsWalker::new(&tangle, ids[&8]);

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    let bfs_matched: Vec<MessageId> = IntoIterator::into_iter([8, 7, 6, 5, 4, 3, 2, 1])
        .map(|node| ids[&node])
        .collect();
    let bfs_missing: Vec<MessageId> = IntoIterator::into_iter([0]).map(|node| ids[&node]).collect();

    for status in walker {
        match status {
            TangleWalkerItem::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerItem::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerItem::Missing(message_id) => missing.push(message_id),
        }
    }

    assert_eq!(bfs_matched, matched);
    assert!(skipped.is_empty());
    assert_eq!(bfs_missing, missing);
}
