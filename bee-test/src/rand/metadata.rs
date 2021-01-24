// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::rand_integer, milestone::rand_milestone_index, option::rand_option};

use bee_tangle::{flags::Flags, metadata::MessageMetadata};

pub fn rand_metadata() -> MessageMetadata {
    MessageMetadata::new(
        unsafe { Flags::from_bits_unchecked(rand_integer::<u8>()) },
        rand_milestone_index(),
        rand_integer(),
        rand_integer(),
        rand_integer(),
        rand_option(rand_milestone_index()),
        rand_option(rand_milestone_index()),
        rand_option(rand_milestone_index()),
        rand_integer(),
    )
}
