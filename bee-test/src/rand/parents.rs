// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    integer::rand_integer_range,
    message::{rand_message_id, rand_message_ids},
};

use bee_message::Parents;

pub fn rand_parents() -> Parents {
    Parents::new(rand_message_id(), rand_message_ids(rand_integer_range(0..8))).unwrap()
}
