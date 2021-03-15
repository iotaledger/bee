// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::ConflictReason;

use bee_message::{
    milestone::MilestoneIndex,
    output::{ConsumedOutput, CreatedOutput},
    MessageId,
};

// TODO why do we need to full vectors here ?
#[derive(Clone)]
pub struct MilestoneConfirmed {
    pub id: MessageId,
    pub index: MilestoneIndex,
    pub timestamp: u64,
    pub referenced_messages: usize,
    pub excluded_no_transaction_messages: Vec<MessageId>,
    pub excluded_conflicting_messages: Vec<(MessageId, ConflictReason)>,
    pub included_messages: Vec<MessageId>,
    pub created_outputs: usize,
    pub consumed_outputs: usize,
}

pub struct NewConsumedOutput(pub ConsumedOutput);

pub struct NewCreatedOutput(pub CreatedOutput);
