// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

#[derive(Debug, Default)]
pub(crate) struct PruningMetrics {
    pub(crate) curr_seps: usize,
    pub(crate) new_seps: usize,
    pub(crate) kept_seps: usize,
    pub(crate) next_seps: usize,
    pub(crate) messages: usize,
    pub(crate) edges: usize,
    pub(crate) indexations: usize,
    // TODO
    // pub(crate) output_diffs: bool,
    pub(crate) receipts: usize,
}

#[derive(Debug, Default)]
pub(crate) struct ConfirmedDataPruningMetrics {
    pub(crate) msg_already_visited: usize,
    pub(crate) references_sep: usize,
    pub(crate) approver_cache_miss: usize,
    pub(crate) approver_cache_hit: usize,
    pub(crate) all_approvers_visited: usize,
    pub(crate) not_all_approvers_visited: usize,
    pub(crate) found_seps: usize,
    pub(crate) prunable_messages: usize,
    pub(crate) prunable_edges: usize,
    pub(crate) prunable_indexations: usize,
    pub(crate) new_seps: usize,
}

#[derive(Debug, Default)]
pub(crate) struct UnconfirmedDataPruningMetrics {
    pub(crate) none_received: bool,
    pub(crate) prunable_messages: usize,
    pub(crate) prunable_edges: usize,
    pub(crate) prunable_indexations: usize,
    pub(crate) already_pruned: usize,
    pub(crate) were_confirmed: usize,
}

#[derive(Debug, Default)]
pub(crate) struct MilestoneDataPruningMetrics {
    pub(crate) receipts: usize,
}

#[derive(Debug, Default)]
pub(crate) struct Timings {
    pub(crate) full_prune: Duration,
    pub(crate) get_curr_seps: Duration,
    pub(crate) filter_curr_seps: Duration,
    pub(crate) replace_seps: Duration,
    pub(crate) batch_confirmed_data: Duration,
    pub(crate) batch_unconfirmed_data: Duration,
    pub(crate) batch_milestone_data: Duration,
    pub(crate) batch_new_seps: Duration,
    pub(crate) truncate_curr_seps: Duration,
    pub(crate) batch_commit: Duration,
}
