// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Opinion statement.

use crate::{error::Error, opinion};

use bee_common::packable::{Packable, Read, Write};

use core::{
    cmp::{Ord, Ordering, PartialOrd},
    ops::Deref,
};
use std::{collections::BinaryHeap, ops::DerefMut};

/// Length (in bytes) of this statement when serialized.
pub const OPINION_STATEMENT_LENGTH: usize = 2;

/// Opinion registry statement.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Opinion {
    /// The opinion of the voting object.
    pub opinion: opinion::Opinion,
    /// The round in which this opinion was formed.
    pub round: u8,
}

impl Ord for Opinion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.round.cmp(&other.round)
    }
}

impl PartialOrd for Opinion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Packable for Opinion {
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
        let opinion = opinion::Opinion::unpack(reader)?;
        let round = <u8>::unpack(reader)?;

        Ok(Self { opinion, round })
    }
}

/// Wrapper struct for a collection of `Opinion` statements.
#[derive(Debug, Clone)]
pub struct Opinions(BinaryHeap<Opinion>);

impl Deref for Opinions {
    type Target = BinaryHeap<Opinion>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Opinions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Opinions {
    /// Create a new, empty `Opinions` collection.
    pub fn new() -> Self {
        Self(BinaryHeap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the `Opinion` that was formed on the most recent round.
    pub fn last(&self) -> Option<&Opinion> {
        self.0.peek()
    }

    /// Check that the `Opinion` at a given index is finalized.
    pub fn finalized(&self, idx: usize) -> bool {
        if idx > self.len() {
            return false;
        }

        let last = self.last();

        // Check for identical consecutive `Opinion`s after the given index.
        if let Some(last) = last {
            for (i, opinion) in self.0.iter().enumerate() {
                if i >= idx && opinion != last {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}
