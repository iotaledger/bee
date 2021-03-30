// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Events that can occur during the vote process, to be transmitted through a channel.

use crate::{VoteObject, context::VoteContext, error, opinion::{Opinion, QueriedOpinions}};

use std::{collections::HashMap, time::Duration};

/// Describes an event that occured during the vote process.
#[derive(Debug)]
pub enum Event {
    /// Error occurred during voting.
    Error(error::Error),
    /// Vote failed.
    Failed(OpinionEvent),
    /// Vote finished and finalized correctly.
    Finalized(OpinionEvent),
    /// Voting round has been successfully executed.
    RoundExecuted(RoundStats),
}

/// Statistics of the round that has just been executed.
#[derive(Debug)]
pub struct RoundStats {
    /// Duration of the voiting round.
    pub duration: Duration,
    /// Random number used in the vote.
    pub rand_used: f64,
    /// Active vote contexts.
    /// Note: this does not contain any contexts that were finalized or failed during the round.
    pub vote_contexts: HashMap<VoteObject, VoteContext>,
    /// Opinions queried during the round.
    pub queried_opinions: Vec<QueriedOpinions>,
}

/// Information to be passed on a Failed or Finalized event.
#[derive(Debug)]
pub struct OpinionEvent {
    /// Object of the voting, and related ID.
    pub object: VoteObject,
    /// The opinion of the conflict.
    pub opinion: Opinion,
    /// Context of the conflict.
    pub context: VoteContext,
}
