// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Context information for the voting pool.

use crate::{Error, Opinion, Opinions};

use bee_message::prelude::{MessageId, TransactionId};

use std::fmt;

/// Initial "liked" value for a new `Context`.
pub const LIKED_INITIAL: f64 = -1.0;

/// Object type of a vote.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VoteObject {
    /// Conflict type and related transaction ID.
    Conflict(TransactionId),
    /// Timestamp type and related message ID.
    Timestamp(MessageId),
}

impl fmt::Display for VoteObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Conflict(id) => write!(f, "Conflict({})", id),
            Self::Timestamp(id) => write!(f, "Timestamp({})", id),
        }
    }
}

impl VoteObject {
    /// Get a `MessageId` if this is a Timestamp object.
    /// Returns `Some(id)` if this is the case, `None` if not.
    pub fn message_id(&self) -> Option<MessageId> {
        if let Self::Timestamp(id) = *self {
            Some(id)
        } else {
            None
        }
    }

    /// Get a `TransactionId` if this is a Conflict object.
    /// Returns `Some(id)` if this is the canse, `None` if not.
    pub fn transaction_id(&self) -> Option<TransactionId> {
        if let Self::Conflict(id) = *self {
            Some(id)
        } else {
            None
        }
    }
}

/// Builder pattern struct for instantiating a `VoteContext`.
pub struct VoteContextBuilder {
    /// Object of the voting (conflict or timestamp), and associated ID.
    object: VoteObject,
    /// Opinions held by this context on the vote object.
    opinions: Option<Opinions>,
}

impl VoteContextBuilder {
    /// Create a new `VoteContextBuilder`, defining an ID and an `ObjectType` (voting object).
    pub fn new(object: VoteObject) -> Self {
        Self { object, opinions: None }
    }

    /// Set a single initial `Opinion`.
    pub fn with_initial_opinion(mut self, opinion: Opinion) -> Self {
        self.opinions = Some(Opinions::new(vec![opinion]));
        self
    }

    /// Set a list of initial `Opinion`s.
    pub fn with_initial_opinions(mut self, opinions: Opinions) -> Self {
        self.opinions = Some(opinions);
        self
    }

    /// Build a `VoteContext`.
    /// Note: this will panic if no initial opinions have been provided.
    pub fn build(self) -> Result<VoteContext, Error> {
        Ok(VoteContext {
            object: self.object,
            liked: LIKED_INITIAL,
            rounds: 0,
            opinions: self.opinions.ok_or(Error::NoInitialOpinions)?,
        })
    }
}

/// Voting context.
#[derive(Debug, Clone)]
pub struct VoteContext {
    /// Object type of the vote and related object ID.
    object: VoteObject,
    /// The percentage of `OpinionGiver`s who liked this item on the last query.
    liked: f64,
    /// The number of voting rounds performed so far.
    rounds: u32,
    /// List of opinions formed at the end of each voting round.
    /// The first in this list is the initial opinion when this `VoteContext` was created.
    opinions: Opinions,
}

impl VoteContext {
    /// Constructs a new `VoteContext`.
    pub(crate) fn new(object: VoteObject, initial_opinion: Opinion) -> Self {
        Self {
            object,
            liked: LIKED_INITIAL,
            rounds: 0,
            opinions: Opinions::new(vec![initial_opinion]),
        }
    }

    /// Add the given `Opinion` to the `VoteContext`.
    pub(crate) fn add_opinion(&mut self, opinion: Opinion) {
        self.opinions.push(opinion);
    }

    /// Retrieve the last formed `Opinion`.
    pub fn last_opinion(&self) -> Option<Opinion> {
        self.opinions.last().copied()
    }

    /// Describes whether this `VoteContext` has been finalized.
    pub fn finalized(&self, cool_off_period: u32, total_rounds_finalization: u32) -> bool {
        // Check whether we have enough opinions to decide if the vote is finalised.
        if self.opinions.len() < (cool_off_period + total_rounds_finalization + 1) as usize {
            false
        } else {
            // Index of the opinion that needs to be held for `finalization_threshold` number of rounds.
            let finalized_index = self.opinions.len() - total_rounds_finalization as usize;

            if self.opinions.len() < finalized_index + 1 {
                return false;
            }

            // Check that this opinion is held.
            if let Some(candidate_opinion) = self.opinions.get(finalized_index) {
                let subsequent_opinions = self.opinions.split_at(finalized_index + 1).1;

                for opinion in subsequent_opinions {
                    if opinion != candidate_opinion {
                        return false;
                    }
                }

                true
            } else {
                false
            }
        }
    }

    /// Describes whether the `VoteContext` is new (has not participated in a vote).
    pub(crate) fn is_new(&self) -> bool {
        self.liked == LIKED_INITIAL
    }

    /// Described whether the `VoteContext` has *just* finished its first round.
    pub(crate) fn had_first_round(&self) -> bool {
        self.rounds == 1
    }

    pub(crate) fn had_fixed_round(
        &self,
        cool_off_period: u32,
        total_rounds_finalization: u32,
        total_rounds_fixed: u32,
    ) -> bool {
        let total_rounds_random_threshold = total_rounds_finalization as i32 - total_rounds_fixed as i32;

        if self.opinions.len() < (cool_off_period as i32 + total_rounds_random_threshold + 1) as usize {
            return false;
        }

        if self.opinions.len() < total_rounds_random_threshold as usize || total_rounds_random_threshold < 0 {
            return false;
        }

        let candidate_idx = self.opinions.len() - total_rounds_random_threshold as usize;
        let candidate_opinion = self.opinions[candidate_idx];

        for i in candidate_idx..self.opinions.len() {
            let subsequent_opinion = self.opinions[i];

            if subsequent_opinion != candidate_opinion {
                return false;
            }
        }

        true
    }

    /// Returns the object of the voting.
    pub fn object(&self) -> VoteObject {
        self.object
    }

    /// Resturns the percentage of `OpinionGiver`s that liked the item on the last query.
    pub fn liked(&self) -> f64 {
        self.liked
    }

    /// Update the `liked` value of a `VoteContext` when new opinions are formed.
    pub(crate) fn set_liked(&mut self, liked: f64) {
        self.liked = liked;
    }

    /// Number of voting rounds completed for this item.
    pub fn rounds(&self) -> u32 {
        self.rounds
    }

    /// Indicate the completion of a voting round for this item.
    pub(crate) fn round_completed(&mut self) {
        self.rounds += 1;
    }
}

mod tests {
    use super::*;
    
    #[test]
    fn had_fixed_round() {
        let ctx = VoteContextBuilder::new(VoteObject::Conflict(TransactionId::new([0u8; 32])))
            .with_initial_opinions(Opinions::new(vec![Opinion::Like; 5]))
            .build()
            .unwrap();

        assert!(ctx.had_fixed_round(2, 4, 2));
    }

    #[test]
    fn not_had_fixed_round() {
        let ctx = VoteContextBuilder::new(VoteObject::Conflict(TransactionId::new([0u8; 32])))
            .with_initial_opinions(Opinions::new(vec![
                Opinion::Like,
                Opinion::Like,
                Opinion::Like,
                Opinion::Like,
                Opinion::Dislike
            ]))
            .build()
            .unwrap();

        assert!(!ctx.had_fixed_round(2, 4, 2));
    }
}
