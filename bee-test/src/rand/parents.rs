// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::parents::{Parents, MESSAGE_PARENTS_RANGE};

use crate::rand::{message::rand_message_ids, number::rand_number_range};

/// Generates random parents.
pub fn rand_parents() -> Parents {
    Parents::new(rand_message_ids(rand_number_range(MESSAGE_PARENTS_RANGE))).unwrap()
}
