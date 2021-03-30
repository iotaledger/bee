// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Functionality for handling opinions.

use crate::{Error, VoteObject};

use bee_common::packable::{Packable, Read, Write};

use std::{collections::HashMap, fmt, ops};

/// Gives `Opinion`s about the given IDs.
pub trait OpinionGiver {
    /// Queries the `OpinionGiver` for its opinions of given IDs.
    fn query(&mut self, objects: &QueryObjects) -> Result<Opinions, Error>;

    /// The ID of the `OpinionGiver`.
    fn id(&self) -> &str;
}

/// Collection of IDs to query for opinions.
pub struct QueryObjects {
    /// IDs that have opinions on conflicts.
    pub conflict_objects: Vec<VoteObject>,
    /// IDs that have opinions on timestamps.
    pub timestamp_objects: Vec<VoteObject>,
}

#[derive(Debug)]
/// Represents `Opinion`s queried from an `OpinionGiver`.
pub struct QueriedOpinions {
    /// ID of the `OpinionGiver`.
    pub opinion_giver_id: String,
    /// Map of IDs to `Opinion`s.
    pub opinions: HashMap<VoteObject, Opinion>,
    /// The amount of times the `OpinionGiver`'s opinion has been counted.
    /// Usually this number is 1, but due to randomisation of the queried `OpinionGiver`s,
    /// the same `OpinionGiver`'s opinions might be counted multiple times.
    pub times_counted: u32,
}

/// Defines an opinion.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opinion {
    /// Defines a "like" opinion.
    Like = 0x01,
    /// Defines a "dislike" opinion.
    Dislike = 0x02,
    /// Defines an "unknown" opinion.
    Unknown = 0x04,
}

impl fmt::Display for Opinion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Packable for Opinion {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (*self as u8).pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Error> {
        Ok(match u8::unpack(reader)? {
            0x01 => Opinion::Like,
            0x02 => Opinion::Dislike,
            0x04 => Opinion::Unknown,
            other => return Err(Self::Error::InvalidOpinion(other)),
        })
    }
}

/// Wrapper tuple struct for a collection of opinions.
#[derive(Clone)]
pub struct Opinions(Vec<Opinion>);

impl fmt::Debug for Opinions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl ops::Deref for Opinions {
    type Target = Vec<Opinion>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Opinions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Opinions {
    /// Create a new `Opinions` wrapper from `Vec<Opinion>`.
    pub fn new(inner: Vec<Opinion>) -> Self {
        Self(inner)
    }

    /// Get the number of `Opinion`s.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Determine if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over inner `Opinion`s.
    pub fn iter(&self) -> impl Iterator<Item = &Opinion> + '_ {
        self.0.iter()
    }
}
