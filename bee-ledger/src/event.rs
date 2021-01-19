// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conflict::ConflictReason,
    model::{Output, Spent},
};

use bee_message::{milestone::MilestoneIndex, MessageId};

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
    pub spent_outputs: usize,
    pub created_outputs: usize,
}

pub struct NewSpent(pub Spent);

pub struct NewOutput(pub Output);
