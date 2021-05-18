// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, time::Duration};

#[derive(Debug, Default)]
pub struct PruningMetrics {
    pub messages: usize,
    pub edges: usize,
    pub indexations: usize,
    pub milestones: usize,
    pub output_diffs: usize,
    pub receipts: usize,
}

#[derive(Debug, Default)]
pub struct TraversalMetrics {
    pub msg_already_visited: usize,
    pub bottomed: usize,
    pub fetched_messages: usize,
    pub fetched_approvers: usize,
    pub buffered_approvers: usize,
    pub all_approvers_visited: usize,
    pub approvers_not_visited: usize,
    pub messages: usize,
    pub edges: usize,
    pub indexations: usize,
    pub found_sep_early: usize,
    pub found_sep_late: usize,
    pub new_seps: usize,
}

#[derive(Default)]
pub struct TimingMetrics {
    pub full_prune: Duration,
    pub get_old_seps: Duration,
    pub replace_seps: Duration,
    pub batch_del_confirmed: Duration,
    pub batch_del_unconfirmed: Duration,
    pub batch_del_milestones: Duration,
    pub batch_del_output_diffs: Duration,
    pub batch_del_receipts: Duration,
    pub batch_ins_new_seps: Duration,
    pub truncate_old_seps: Duration,
    pub batch_commit: Duration,
}

impl fmt::Debug for TimingMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PruningMetrics")
            .field("full_prune", &self.full_prune.as_secs_f32())
            .field("get_old_seps", &self.get_old_seps.as_secs_f32())
            .field("replace_seps", &self.replace_seps.as_secs_f32())
            .field("batch_del_confirmed", &self.batch_del_confirmed.as_secs_f32())
            .field("batch_del_unconfirmed", &self.batch_del_unconfirmed.as_secs_f32())
            .field("batch_del_milestones", &self.batch_del_milestones.as_secs_f32())
            .field("batch_del_output_diffs", &self.batch_del_output_diffs.as_secs_f32())
            .field("batch_del_receipts", &self.batch_del_receipts.as_secs_f32())
            .field("batch_ins_new_seps", &self.batch_ins_new_seps.as_secs_f32())
            .field("truncate_old_seps", &self.truncate_old_seps.as_secs_f32())
            .field("batch_commit", &self.batch_commit.as_secs_f32())
            .finish()
    }
}
