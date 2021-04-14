// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer_range, message::rand_message_ids};

use bee_message::parents::{Parents, MESSAGE_PARENTS_RANGE};

/// Generates random parents.
pub fn rand_parents() -> Parents {
    Parents::new(rand_message_ids(rand_integer_range(MESSAGE_PARENTS_RANGE))).unwrap()
}
