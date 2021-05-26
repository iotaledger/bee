// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

#[derive(Debug, Default)]
pub struct PruningMetrics {
    pub old_seps: usize,
    pub found_seps: usize,
    pub kept_seps: usize,
    pub new_seps: usize,
    pub messages: usize,
    pub edges: usize,
    pub indexations: usize,
    pub milestones: usize,
    pub output_diffs: usize,
    pub receipts: usize,
}

#[derive(Debug, Default)]
pub struct ConfirmedMetrics {
    pub msg_already_visited: usize,
    pub bottomed: usize,
    pub fetched_messages: usize,
    pub fetched_approvers: usize,
    pub buffered_approvers: usize,
    pub all_approvers_visited: usize,
    pub approvers_not_visited: usize,
    pub found_sep_early: usize,
    pub found_sep_late: usize,
    pub prunable_messages: usize,
    pub prunable_edges: usize,
    pub prunable_indexations: usize,
    pub new_seps: usize,
}

#[derive(Debug, Default)]
pub struct UnconfirmedMetrics {
    pub no_unconfirmed: usize,
    pub prunable_messages: usize,
    pub prunable_edges: usize,
    pub prunable_indexations: usize,
    pub already_pruned: usize,
}

#[derive(Debug, Default)]
pub struct Timings {
    pub full_prune: Duration,
    pub get_old_seps: Duration,
    pub filter_old_seps: Duration,
    pub replace_seps: Duration,
    pub batch_confirmed: Duration,
    pub batch_unconfirmed: Duration,
    pub batch_milestones: Duration,
    pub batch_output_diffs: Duration,
    pub batch_receipts: Duration,
    pub batch_new_seps: Duration,
    pub truncate_old_seps: Duration,
    pub batch_commit: Duration,
}
