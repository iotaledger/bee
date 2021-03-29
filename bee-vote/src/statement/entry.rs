// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::opinion::{Opinion, Opinions};

use core::{
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct Entry {
    pub opinions: Opinions,
    pub timestamp: u64,
}

pub trait EntryType {
    type Id;

    fn id(&self) -> &Self::Id;
    fn opinion(&self) -> &Opinion;
}

#[derive(Debug)]
pub struct EntryMap<I, T> {
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
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            phantom: PhantomData,
        }
    }

    pub fn add_entry(&mut self, entry: T) {
        if !self.contains_key(entry.id()) {
            let mut opinions = Opinions::new();
            opinions.push(*entry.opinion());

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
            existing_entry.opinions.push(*entry.opinion());
        }
    }

    pub fn add_entries(&mut self, entries: Vec<T>) {
        for entry in entries.into_iter() {
            self.add_entry(entry);
        }
    }

    pub fn get_entry_opinions(&self, id: &I) -> Option<Opinions> {
        self.deref().get(id).map(|entry| entry.opinions.clone())
    }
}
