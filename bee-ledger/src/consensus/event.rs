// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{ConsumedOutput, CreatedOutput};

use bee_ledger_types::types::ConflictReason;
use bee_message::{milestone::MilestoneIndex, MessageId};

#[derive(Clone)]
pub struct MilestoneConfirmed {
    pub id: MessageId,
    pub index: MilestoneIndex,
    pub timestamp: u64,
    pub referenced_messages: usize,
    pub excluded_no_transaction_messages: Vec<MessageId>,
    pub excluded_conflicting_messages: Vec<(MessageId, ConflictReason)>,
    pub included_messages: Vec<MessageId>,
    pub consumed_outputs: usize,
    pub created_outputs: usize,
    pub receipt: bool,
}

pub struct OutputConsumed(pub ConsumedOutput);

pub struct OutputCreated(pub CreatedOutput);

pub struct SnapshottedIndex(pub MilestoneIndex);

pub struct PrunedIndex(pub MilestoneIndex);
