// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, time::Duration};

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

#[derive(Default)]
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

impl fmt::Debug for Timings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PruningMetrics")
            .field("full_prune", &self.full_prune.as_secs_f32())
            .field("get_old_seps", &self.get_old_seps.as_secs_f32())
            .field("filter_old_seps", &self.filter_old_seps.as_secs_f32())
            .field("replace_seps", &self.replace_seps.as_secs_f32())
            .field("batch_confirmed", &self.batch_confirmed.as_secs_f32())
            .field("batch_unconfirmed", &self.batch_unconfirmed.as_secs_f32())
            .field("batch_milestones", &self.batch_milestones.as_secs_f32())
            .field("batch_output_diffs", &self.batch_output_diffs.as_secs_f32())
            .field("batch_receipts", &self.batch_receipts.as_secs_f32())
            .field("batch_new_seps", &self.batch_new_seps.as_secs_f32())
            .field("truncate_old_seps", &self.truncate_old_seps.as_secs_f32())
            .field("batch_commit", &self.batch_commit.as_secs_f32())
            .finish()
    }
}
