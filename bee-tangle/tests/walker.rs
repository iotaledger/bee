// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use bee_message::MessageId;
use bee_tangle::{TangleWalker, TangleWalkerStatus};

#[test]
fn walk() {
    // 0 --
    //     | -- 8 --
    // 1 --         |
    //              | -- 12 --
    // 2 --         |          |
    //     | -- 9 --           |
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
        8 => 0, 1;
        9 => 2, 3;
        10 => 4, 5;
        11 => 6, 7;
        12 => 8, 9;
        13 => 10, 11;
        14 => 12, 13;
    };

    let correct_order: Vec<MessageId> = IntoIterator::into_iter([14, 12, 8, 0, 1, 9, 2, 3, 13, 10, 4, 5, 11, 6, 7])
        .map(|node| ids[&node])
        .collect();

    let mut matched = Vec::new();
    let mut skipped = Vec::new();
    let mut missing = Vec::new();

    let mut walker = TangleWalker::new(&tangle, ids[&14]);

    while let Some(status) = walker.next() {
        match status {
            TangleWalkerStatus::Matched(message_id, message_data) => matched.push(message_id),
            TangleWalkerStatus::Skipped(message_id, message_data) => skipped.push(message_id),
            TangleWalkerStatus::Missing(message_id) => missing.push(message_id),
        }
    }

    assert_eq!(correct_order, matched);
    assert!(skipped.is_empty());
    assert_eq!(missing, vec![MessageId::null()]);
}
