// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    conflict::Conflict,
    entry::{Entry, EntryMap},
    opinion::Opinions,
    timestamp::Timestamp,
};
use crate::{
    opinion::{self, QueryIds},
    Error,
};

use bee_message::{payload::transaction::TransactionId, MessageId};

use tokio::sync::RwLock;

use core::str::FromStr;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

// TODO String -> Node ID
#[derive(Debug)]
pub struct View {
    pub node_id: String,
    pub conflicts: EntryMap<TransactionId, Conflict>,
    pub timestamps: EntryMap<MessageId, Timestamp>,
}

// TODO clean-up (generics)
impl View {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            conflicts: EntryMap::new(),
            timestamps: EntryMap::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.node_id
    }

    pub fn add_conflict(&mut self, conflict: Conflict) {
        self.conflicts.add_entry(conflict);
    }

    pub fn add_conflicts(&mut self, conflicts: Vec<Conflict>) {
        self.conflicts.add_entries(conflicts);
    }

    pub fn add_timestamp(&mut self, timestamp: Timestamp) {
        self.timestamps.add_entry(timestamp);
    }

    pub fn add_timestamps(&mut self, timestamps: Vec<Timestamp>) {
        self.timestamps.add_entries(timestamps);
    }

    pub fn get_conflict_opinions(&self, id: TransactionId) -> Option<Opinions> {
        self.conflicts.get_entry_opinions(&id)
    }

    pub fn get_timestamp_opinions(&self, id: MessageId) -> Option<Opinions> {
        self.timestamps.get_entry_opinions(&id)
    }

    pub fn query(&mut self, query_ids: &QueryIds) -> Result<opinion::Opinions, Error> {
        // TODO default empty `Opinions`.
        let mut opinions = opinion::Opinions::new(vec![]);

        for id in query_ids.conflict_ids.iter() {
            if let Some(conflict_opinions) = self.get_conflict_opinions(TransactionId::from_str(id)?) {
                if !conflict_opinions.is_empty() {
                    // This will never fail.
                    opinions.push(conflict_opinions.last().unwrap().opinion);
                } else {
                    opinions.push(opinion::Opinion::Unknown);
                }
            } else {
                opinions.push(opinion::Opinion::Unknown);
            };
        }

        for id in query_ids.timestamp_ids.iter() {
            if let Some(timestamp_opinions) = self.get_timestamp_opinions(MessageId::from_str(id)?) {
                if !timestamp_opinions.is_empty() {
                    // This will never fail.
                    opinions.push(timestamp_opinions.last().unwrap().opinion);
                } else {
                    opinions.push(opinion::Opinion::Unknown);
                }
            } else {
                opinions.push(opinion::Opinion::Unknown);
            };
        }

        Ok(opinions)
    }
}

// TODO String -> Node ID
#[derive(Default)]
pub struct Registry {
    pub views: RwLock<HashMap<String, View>>,
}

impl Registry {
    pub async fn write_view(&self, node_id: &String, f: impl FnOnce(&mut View)) {
        let mut guard = self.views.write().await;

        if !guard.contains_key(node_id) {
            guard.insert(
                node_id.to_string(),
                View {
                    node_id: node_id.to_string(),
                    conflicts: EntryMap::new(),
                    timestamps: EntryMap::new(),
                },
            );
        }

        f(guard.get_mut(node_id).unwrap());
    }

    pub async fn read_view(&self, node_id: &String, f: impl FnOnce(&View)) -> Result<(), Error> {
        let guard = self.views.read().await;
        let view = guard.get(node_id).ok_or(Error::NodeNotFound(node_id.to_string()))?;
        f(view);
        Ok(())
    }

    pub async fn clean(&self, duration: Duration) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;

        let mut guard = self.views.write().await;
        for (_, view) in guard.iter_mut() {
            let filter = |entry: &Entry| -> bool { now - entry.timestamp < duration.as_millis() as u64 };

            (*view.conflicts).retain(|_, entry| filter(entry));
            view.timestamps.retain(|_, entry| filter(entry));
        }
    }
}
