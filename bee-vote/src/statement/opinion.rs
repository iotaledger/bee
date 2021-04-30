// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! OpinionStatement statement.

use crate::{error::Error, Opinion};

use bee_common::packable::{Packable, Read, Write};

use std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::BTreeSet,
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

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let opinion = Opinion::unpack_inner::<R, CHECK>(reader)?;
        let round = <u8>::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self { opinion, round })
    }
}

/// Wrapper struct for a collection of `OpinionStatement` statements.
#[derive(Debug, Clone)]
pub struct OpinionStatements(BTreeSet<OpinionStatement>);

impl OpinionStatements {
    /// Create a new, empty `OpinionStatements` collection.
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the `OpinionStatement` that was formed on the most recent round.
    pub fn last(&self) -> Option<&OpinionStatement> {
        self.0.iter().last()
    }

    /// Insert an `OpinionStatement` into the collection. An error will be returned if the
    /// `OpinionStatement` already exists.
    pub fn insert(&mut self, statement: OpinionStatement) -> Result<(), Error> {
        if !self.0.insert(statement) {
            Err(Error::DuplicateOpinionStatement(statement))
        } else {
            Ok(())
        }
    }

    /// Clear the collection.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Check that the `OpinionStatement` at a given index is finalized.
    pub fn finalized(&self, idx: usize) -> bool {
        if idx > self.len() {
            return false;
        }

        let last = self.last();

        // Check for identical consecutive `Opinion`s after the given index.
        if let Some(last) = last {
            for (i, opinion_statement) in self.0.iter().enumerate() {
                if i >= idx && opinion_statement.opinion != last.opinion {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}
