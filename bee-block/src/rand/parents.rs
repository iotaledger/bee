// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parent::Parents,
    rand::{block::rand_block_ids, number::rand_number_range},
};

/// Generates random parents.
pub fn rand_parents() -> Parents {
    Parents::new(rand_block_ids(rand_number_range(Parents::COUNT_RANGE).into())).unwrap()
}
