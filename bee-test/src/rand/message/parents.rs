// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    message::rand_message_id,
    number::{rand_number, rand_number_range},
    vec::rand_vec,
};

use bee_message::{
    parents::{Parent, Parents, MESSAGE_PARENTS_RANGE},
    MessageId,
};

/// Generates a random [`Parent`].
pub fn rand_parent() -> Parent {
    match rand_number::<u8>() % 2 {
        0 => Parent::Strong(rand_message_id()),
        1 => Parent::Weak(rand_message_id()),
        _ => unreachable!(),
    }
}

/// Generates a random [`Parents`].
pub fn rand_parents() -> Parents {
    let mut parents = vec![Parent::Strong(rand_message_id())];

    parents.extend(rand_vec(
        rand_parent,
        (rand_number_range(MESSAGE_PARENTS_RANGE) - 1).into(),
    ));

    parents.sort();

    Parents::new(parents).unwrap()
}

/// Creates a [`Parents`] from a [`Vec<MessageId>`].
pub fn parents_from_ids(message_ids: Vec<MessageId>) -> Parents {
    let mut parents = Vec::with_capacity(message_ids.len());
    parents.push(Parent::Strong(message_ids[0]));

    for message_id in message_ids.into_iter().skip(1) {
        parents.push(match rand_number::<u8>() % 2 {
            0 => Parent::Strong(message_id),
            1 => Parent::Weak(message_id),
            _ => unreachable!(),
        });
    }

    parents.sort();

    Parents::new(parents).unwrap()
}
