// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_message::MessageId;
use bee_tangle::walkers::{TangleBfsWalker, TangleDfsWalker, TangleWalkerStatus};

use std::collections::{HashMap, VecDeque};

#[test]
fn walk() {
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

    let (tangle, ids, graph) = tangle! {
        8 => [0, 1]
        9 => [2, 3]
        10 => [4, 5]
        11 => [6, 7]
        12 => [8, 9]
        13 => [10, 11]
        14 => [12, 13]
    };

    let start_id = ids[&14];

    let mut dfs_order = Vec::with_capacity(graph.len());

    fn dfs(node: MessageId, graph: &HashMap<MessageId, Vec<MessageId>>, order: &mut Vec<MessageId>) {
        if let Some(parents) = graph.get(&node) {
            order.push(node);
            for &parent in parents.iter().rev() {
                dfs(parent, graph, order);
            }
        }
    }

    dfs(start_id, &graph, &mut dfs_order);

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    let walker = TangleDfsWalker::new(&tangle, start_id);

    for status in walker {
        match status {
            TangleWalkerStatus::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerStatus::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerStatus::Missing(message_id) => missing.push(message_id),
        }
    }

    let mut correct_missing = (0..=7).map(|node| ids[&node]).collect::<Vec<MessageId>>();

    correct_missing.sort();
    missing.sort();

    assert_eq!(dfs_order, matched);
    assert!(skipped.is_empty());
    assert_eq!(correct_missing, missing);

    let mut bfs_order = Vec::with_capacity(graph.len());

    fn bfs(node: MessageId, graph: &HashMap<MessageId, Vec<MessageId>>, order: &mut Vec<MessageId>) {
        let mut queue = VecDeque::new();
        queue.push_back(node);

        while let Some(node) = queue.pop_front() {
            if let Some(parents) = graph.get(&node) {
                order.push(node);
                for &parent in parents {
                    queue.push_back(parent);
                }
            }
        }
    }

    bfs(start_id, &graph, &mut bfs_order);

    matched.clear();
    skipped.clear();
    missing.clear();

    let walker = TangleBfsWalker::new(&tangle, start_id);

    for status in walker {
        match status {
            TangleWalkerStatus::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerStatus::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerStatus::Missing(message_id) => missing.push(message_id),
        }
    }

    missing.sort();

    dbg!(ids);
    assert_eq!(bfs_order, matched);
    assert!(skipped.is_empty());
    assert_eq!(correct_missing, missing);
}
