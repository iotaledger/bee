// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{message::rand_message_ids, number::rand_number_range};

use bee_message::parent::Parents;

/// Generates random parents.
pub fn rand_parents() -> Parents {
    Parents::new(rand_message_ids(rand_number_range(Parents::COUNT_RANGE))).unwrap()
}
