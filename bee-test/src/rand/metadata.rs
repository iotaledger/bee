// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    integer::rand_integer, message::rand_message_id, milestone::rand_milestone_index, option::rand_option,
};

use bee_ledger_types::types::ConflictReason;
use bee_tangle::{
    flags::Flags,
    metadata::{IndexId, MessageMetadata},
};

pub fn rand_metadata() -> MessageMetadata {
    MessageMetadata::new(
        unsafe { Flags::from_bits_unchecked(rand_integer::<u8>()) },
        rand_option(rand_milestone_index()),
        rand_integer(),
        rand_integer(),
        rand_integer(),
        rand_option(IndexId::new(rand_milestone_index(), rand_message_id())),
        rand_option(IndexId::new(rand_milestone_index(), rand_message_id())),
        // TODO random conflict
        ConflictReason::None,
    )
}
