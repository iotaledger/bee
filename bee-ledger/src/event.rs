// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::{Output, Spent};

use bee_message::MessageId;
use bee_protocol::MilestoneIndex;

#[derive(Clone)]
pub struct MilestoneConfirmed {
    pub id: MessageId,
    pub index: MilestoneIndex,
    pub timestamp: u64,
    pub referenced_messages: usize,
    pub excluded_no_transaction_messages: usize,
    pub excluded_conflicting_messages: usize,
    pub included_messages: Vec<MessageId>,
    pub excluded_messages: Vec<MessageId>,
    pub spent_outputs: usize,
    pub created_outputs: usize,
}

pub struct NewSpent(pub Spent);

pub struct NewOutput(pub Output);
