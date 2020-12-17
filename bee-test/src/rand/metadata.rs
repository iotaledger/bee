// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{integer::random_integer, milestone::random_milestone_index, option::random_option};

use bee_protocol::tangle::{flags::Flags, MessageMetadata};

pub fn random_metadata() -> MessageMetadata {
    MessageMetadata::new(
        unsafe { Flags::from_bits_unchecked(random_integer::<u8>()) },
        random_milestone_index(),
        random_integer::<u64>(),
        random_integer::<u64>(),
        random_integer::<u64>(),
        random_option(random_milestone_index()),
        random_option(random_milestone_index()),
        random_option(random_milestone_index()),
    )
}
