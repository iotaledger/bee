// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};
use bee_tangle::{Tangle, TangleWalker, TangleWalkerStatus};
use bee_test::rand::message::{metadata::rand_message_metadata, rand_message_with_parents_ids};

fn new_sep(tangle: &Tangle) -> (Message, MessageMetadata, MessageId) {
    new_node(tangle, vec![MessageId::null()])
}

fn new_node(tangle: &Tangle, parents_ids: Vec<MessageId>) -> (Message, MessageMetadata, MessageId) {
    let message = rand_message_with_parents_ids(parents_ids);
    let metadata = rand_message_metadata();
    let message_id = message.id();

    tangle.insert(message_id, message.clone(), metadata.clone());

    (message, metadata, message_id)
}

#[test]
fn walk() {
    let tangle = Tangle::new();

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

    let (message_0, metadata_0, message_id_0) = new_sep(&tangle);
    let (message_1, metadata_1, message_id_1) = new_sep(&tangle);
    let (message_2, metadata_2, message_id_2) = new_sep(&tangle);
    let (message_3, metadata_3, message_id_3) = new_sep(&tangle);
    let (message_4, metadata_4, message_id_4) = new_sep(&tangle);
    let (message_5, metadata_5, message_id_5) = new_sep(&tangle);
    let (message_6, metadata_6, message_id_6) = new_sep(&tangle);
    let (message_7, metadata_7, message_id_7) = new_sep(&tangle);

    let (message_8, metadata_8, message_id_8) = new_node(&tangle, vec![message_id_0, message_id_1]);
    let (message_9, metadata_9, message_id_9) = new_node(&tangle, vec![message_id_2, message_id_3]);
    let (message_10, metadata_10, message_id_10) = new_node(&tangle, vec![message_id_4, message_id_5]);
    let (message_11, metadata_11, message_id_11) = new_node(&tangle, vec![message_id_6, message_id_7]);

    let (message_12, metadata_12, message_id_12) = new_node(&tangle, vec![message_id_8, message_id_9]);
    let (message_13, metadata_13, message_id_13) = new_node(&tangle, vec![message_id_10, message_id_11]);

    let (message_14, metadata_14, message_id_14) = new_node(&tangle, vec![message_id_12, message_id_13]);

    let correct_order = vec![
        message_id_14,
        message_id_12,
        message_id_8,
        message_id_0,
        message_id_1,
        message_id_9,
        message_id_2,
        message_id_3,
        message_id_13,
        message_id_10,
        message_id_4,
        message_id_5,
        message_id_11,
        message_id_6,
        message_id_7,
    ];
    let mut traversed_order = Vec::with_capacity(correct_order.len());

    let mut walker = TangleWalker::new(&tangle, message_id_14);

    while let Some(status) = walker.next() {
        if let TangleWalkerStatus::Matched(message_id, message_data) = status {
            traversed_order.push(message_id);
        } else {
            println!("{:?}", status);
        }
    }

    // assert_eq!(correct_order, traversed_order);
}
