// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! OpinionStatement statement.

use crate::{error::Error, Opinion};

use bee_common::packable::{Packable, Read, Write};

use std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::BinaryHeap,
    ops::{Deref, DerefMut},
};

/// Length (in bytes) of this statement when serialized.
pub const OPINION_STATEMENT_LENGTH: usize = 2;

/// OpinionStatement registry statement.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OpinionStatement {
    /// The `Opinion` of the voting object.
    pub opinion: Opinion,
    /// The round in which this OpinionStatement was formed.
    pub round: u8,
}

impl Ord for OpinionStatement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.round.cmp(&other.round)
    }
}

impl PartialOrd for OpinionStatement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Packable for OpinionStatement {
    type Error = Error;

    fn packed_len(&self) -> usize {
        OPINION_STATEMENT_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.opinion.pack(writer)?;
        self.round.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let opinion = Opinion::unpack(reader)?;
        let round = <u8>::unpack(reader)?;

        Ok(Self { opinion, round })
    }
}

/// Wrapper struct for a collection of `OpinionStatement` statements.
#[derive(Debug, Clone)]
pub struct OpinionStatements(BinaryHeap<OpinionStatement>);

impl Deref for OpinionStatements {
    type Target = BinaryHeap<OpinionStatement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OpinionStatements {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OpinionStatements {
    /// Create a new, empty `OpinionStatements` collection.
    pub fn new() -> Self {
        Self(BinaryHeap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the `OpinionStatement` that was formed on the most recent round.
    pub fn last(&self) -> Option<&OpinionStatement> {
        self.0.peek()
    }

    /// Check that the `OpinionStatement` at a given index is finalized.
    pub fn finalized(&self, idx: usize) -> bool {
        if idx > self.len() {
            return false;
        }

        let last = self.last();

        // Check for identical consecutive `OpinionStatement`s after the given index.
        if let Some(last) = last {
            for (i, opinion_statement) in self.0.iter().enumerate() {
                if i >= idx && opinion_statement != last {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}
