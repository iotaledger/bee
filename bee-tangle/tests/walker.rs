// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_message::MessageId;
use bee_tangle::walkers::{TangleBfsWalker, TangleDfsWalker, TangleWalkerItem};

fn generic_walker_test<T: Iterator<Item = TangleWalkerItem>>(
    walker: T,
    expected_matched: Vec<MessageId>,
    expected_skipped: Vec<MessageId>,
    expected_missing: Vec<MessageId>,
) {
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
        8  => [0, 1]
        9  => [2, 3]
        10 => [4, 5]
        11 => [6, 7]
        12 => [8, 9]
        13 => [10, 11]
        14 => [12, 13]
    };

    let walker = TangleDfsWalker::new(&tangle, ids[&14]);
    let dfs_matched = IntoIterator::into_iter([14, 12, 8, 9, 13, 10, 11])
        .map(|node| ids[&node])
        .collect();
    let dfs_skipped = Vec::new();
    let dfs_missing = IntoIterator::into_iter([0, 1, 2, 3, 4, 5, 6, 7])
        .map(|node| ids[&node])
        .collect();

    generic_walker_test(walker, dfs_matched, dfs_skipped, dfs_missing);

    let walker = TangleBfsWalker::new(&tangle, ids[&14]);
    let bfs_matched = IntoIterator::into_iter([14, 12, 13, 8, 9, 10, 11])
        .map(|node| ids[&node])
        .collect();
    let bfs_skipped = Vec::new();
    let bfs_missing = IntoIterator::into_iter([0, 1, 2, 3, 4, 5, 6, 7])
        .map(|node| ids[&node])
        .collect();

    generic_walker_test(walker, bfs_matched, bfs_skipped, bfs_missing);
}

#[test]
fn tangle_no_condition() {
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

    let walker = TangleDfsWalker::new(&tangle, ids[&15]);
    let dfs_matched = IntoIterator::into_iter([15, 11, 7, 8, 13, 6, 10, 14, 12, 9])
        .map(|node| ids[&node])
        .collect();
    let dfs_skipped = Vec::new();
    let dfs_missing = IntoIterator::into_iter([3, 1, 2, 4, 0, 5])
        .map(|node| ids[&node])
        .collect();

    generic_walker_test(walker, dfs_matched, dfs_skipped, dfs_missing);

    let walker = TangleBfsWalker::new(&tangle, ids[&15]);
    let bfs_matched = IntoIterator::into_iter([15, 11, 13, 14, 7, 8, 6, 10, 12, 9])
        .map(|node| ids[&node])
        .collect();
    let bfs_skipped = Vec::new();
    let bfs_missing = IntoIterator::into_iter([3, 1, 2, 4, 0, 5])
        .map(|node| ids[&node])
        .collect();

    generic_walker_test(walker, bfs_matched, bfs_skipped, bfs_missing);
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
    let dfs_matched = IntoIterator::into_iter([8, 7, 6, 5, 4, 3, 2, 1])
        .map(|node| ids[&node])
        .collect();
    let dfs_skipped = Vec::new();
    let dfs_missing = IntoIterator::into_iter([0]).map(|node| ids[&node]).collect();

    generic_walker_test(walker, dfs_matched, dfs_skipped, dfs_missing);

    let walker = TangleBfsWalker::new(&tangle, ids[&8]);
    let bfs_matched = IntoIterator::into_iter([8, 7, 6, 5, 4, 3, 2, 1])
        .map(|node| ids[&node])
        .collect();
    let bfs_skipped = Vec::new();
    let bfs_missing = IntoIterator::into_iter([0]).map(|node| ids[&node]).collect();

    generic_walker_test(walker, bfs_matched, bfs_skipped, bfs_missing);
}
