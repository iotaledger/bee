// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_message::MessageId;
use bee_tangle::walkers::{TangleDfsWalker, TangleWalkerStatus};

use std::collections::HashMap;

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
        8 => 0, 1;
        9 => 2, 3;
        10 => 4, 5;
        11 => 6, 7;
        12 => 8, 9;
        13 => 10, 11;
        14 => 12, 13;
    };

    let start_id = ids[&14];

    let mut correct_order = Vec::with_capacity(graph.len());

    fn dfs(node: MessageId, graph: &HashMap<MessageId, Vec<MessageId>>, order: &mut Vec<MessageId>) {
        order.push(node);
        for &parent in graph[&node].iter().rev() {
            dfs(parent, graph, order);
        }
    }

    dfs(start_id, &graph, &mut correct_order);

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    let mut walker = TangleDfsWalker::new(&tangle, start_id);

    for status in walker {
        match status {
            TangleWalkerStatus::Matched(message_id, _message_data) => matched.push(message_id),
            TangleWalkerStatus::Skipped(message_id, _message_data) => skipped.push(message_id),
            TangleWalkerStatus::Missing(message_id) => missing.push(message_id),
        }
    }

    assert_eq!(correct_order, matched);
    assert!(skipped.is_empty());
    assert_eq!(missing, vec![MessageId::null()]);
}
