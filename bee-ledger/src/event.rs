// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::{Output, Spent};

use bee_message::MessageId;
use bee_tangle::milestone::MilestoneIndex;

#[derive(Clone)]
pub struct MilestoneConfirmed {
    pub id: MessageId,
    pub index: MilestoneIndex,
    pub timestamp: u64,
    pub referenced_messages: usize,
    pub excluded_no_transaction_messages: Vec<MessageId>,
    pub excluded_conflicting_messages: Vec<MessageId>,
    pub included_messages: Vec<MessageId>,
    pub spent_outputs: usize,
    pub created_outputs: usize,
}

pub struct NewSpent(pub Spent);

pub struct NewOutput(pub Output);
