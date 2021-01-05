// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::MilestoneIndex;

pub struct MilestoneConfirmed {
    pub index: MilestoneIndex,
    pub timestamp: u64,
    pub referenced_messages: usize,
    pub excluded_no_transaction_messages: usize,
    pub excluded_conflicting_messages: usize,
    pub included_messages: usize,
}
