// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Helper functionality for registry entries.

use crate::Error;
use super::opinion::{OpinionStatement, OpinionStatements};

use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    time::{SystemTime, UNIX_EPOCH},
};

/// Registry entry, containing a list of opinions on the item, and a creation timestamp.
#[derive(Debug, Clone)]
pub(super) struct Entry {
    /// Opinions held on the voting object.
    pub opinions: OpinionStatements,
    /// Time at which the entry was created.
    pub timestamp: u64,
}

/// Trait for types that can be added to an registry `Entry`.
pub(super) trait EntryType {
    /// ID type.
    type Id;

    /// Returns the ID of the entry.
    fn id(&self) -> &Self::Id;
    /// Returns the opinion to be added to the registry `Entry`.
    fn opinion(&self) -> &OpinionStatement;
}

/// `HashMap` of entries, indexed by IDs.
#[derive(Debug)]
pub(super) struct EntryMap<I, T> {
    map: HashMap<I, Entry>,
    phantom: PhantomData<T>,
}

impl<I, T> Deref for EntryMap<I, T>
where
    I: Hash + Eq + PartialEq + Clone,
    T: EntryType<Id = I>,
{
    type Target = HashMap<I, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<I, T> DerefMut for EntryMap<I, T>
where
    I: Hash + Eq + PartialEq + Clone,
    T: EntryType<Id = I>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<I, T> EntryMap<I, T>
where
    I: Hash + Eq + PartialEq + Clone,
    T: EntryType<Id = I>,
{
    /// Create a new, empty `EntryMap`.
    pub(super) fn new() -> Self {
        Self {
            map: HashMap::new(),
            phantom: PhantomData,
        }
    }

    /// Adds an `Entry` to the map.
    /// If an `Entry` with this ID already exists, add the opinion of the given `EntryType` to its stored opinions.
    pub(super) fn add_entry(&mut self, entry: T) -> Result<(), Error> {
        if !self.contains_key(entry.id()) {
            let mut opinions = OpinionStatements::new();
            opinions.insert(*entry.opinion())?;

            self.insert(
                entry.id().clone(),
                Entry {
                    opinions,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Clock may have gone backwards")
                        .as_millis() as u64,
                },
            );
        } else {
            // This will never fail.
            let existing_entry = self.get_mut(entry.id()).unwrap();
            existing_entry.opinions.insert(*entry.opinion())?;
        }

        Ok(())
    }

    /// Add multiple entries to the map.
    pub(super) fn add_entries(&mut self, entries: Vec<T>) -> Result<(), Error> {
        for entry in entries.into_iter() {
            self.add_entry(entry)?;
        }

        Ok(())
    }

    /// Get all the opinions on a given `Entry`.
    pub(super) fn get_entry_opinions(&self, id: &I) -> Option<OpinionStatements> {
        self.deref().get(id).map(|entry| entry.opinions.clone())
    }
}
