// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

#[derive(Debug, Default)]
pub struct PruningMetrics {
    pub curr_seps: usize,
    pub new_seps: usize,
    pub kept_seps: usize,
    pub next_seps: usize,
    pub messages: usize,
    pub edges: usize,
    pub indexations: usize,
    pub output_diffs: bool,
    pub receipts: usize,
}

#[derive(Debug, Default)]
pub struct ConfirmedDataPruningMetrics {
    pub msg_already_visited: usize,
    pub references_sep: usize,
    pub approver_cache_miss: usize,
    pub approver_cache_hit: usize,
    pub all_approvers_visited: usize,
    pub not_all_approvers_visited: usize,
    pub found_seps: usize,
    pub prunable_messages: usize,
    pub prunable_edges: usize,
    pub prunable_indexations: usize,
    pub new_seps: usize,
}

#[derive(Debug, Default)]
pub struct UnconfirmedDataPruningMetrics {
    pub none_received: bool,
    pub prunable_messages: usize,
    pub prunable_edges: usize,
    pub prunable_indexations: usize,
    pub already_pruned: usize,
    pub were_confirmed: usize,
}

#[derive(Debug, Default)]
pub struct MilestoneDataPruningMetrics {
    pub receipts: usize,
}

#[derive(Debug, Default)]
pub struct Timings {
    pub full_prune: Duration,
    pub get_curr_seps: Duration,
    pub filter_curr_seps: Duration,
    pub replace_seps: Duration,
    pub batch_confirmed_data: Duration,
    pub batch_unconfirmed_data: Duration,
    pub batch_milestone_data: Duration,
    pub batch_new_seps: Duration,
    pub truncate_curr_seps: Duration,
    pub batch_commit: Duration,
}
