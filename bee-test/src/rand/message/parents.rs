// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    message::rand_message_id,
    number::{rand_number, rand_number_range},
    vec::rand_vec,
};

use bee_message::parents::{Parent, Parents, PREFIXED_PARENTS_LENGTH_MAX};

/// Generates random parents.
pub fn rand_parents() -> Parents {
    let mut parents_vec = vec![Parent::Strong(rand_message_id())];
    parents_vec.extend(rand_vec(
        rand_parent,
        rand_number_range(0..=PREFIXED_PARENTS_LENGTH_MAX as usize - 1),
    ));

    Parents::new(parents_vec).unwrap()
}

/// Generates a random parent.
pub fn rand_parent() -> Parent {
    match rand_number::<u8>() % 2 {
        0 => Parent::Strong(rand_message_id()),
        1 => Parent::Weak(rand_message_id()),
        _ => unreachable!(),
    }
}
