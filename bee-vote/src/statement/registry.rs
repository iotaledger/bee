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
use bee_network::PeerId;

use tokio::sync::RwLock;

use core::str::FromStr;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// View of all objects that a node has voted on.
#[derive(Debug)]
pub struct View {
    /// ID of the node.
    node_id: PeerId,
    /// Opinions held on transaction conflicts.
    conflicts: EntryMap<TransactionId, Conflict>,
    /// Opinions held on message timestamps.
    timestamps: EntryMap<MessageId, Timestamp>,
}

impl View {
    /// Create a new, empty `View` given a node ID.
    pub fn new(node_id: PeerId) -> Self {
        Self {
            node_id,
            conflicts: EntryMap::new(),
            timestamps: EntryMap::new(),
        }
    }

    /// Get the node ID of the `View`.
    pub fn id(&self) -> &PeerId {
        &self.node_id
    }

    /// Add a conflict entry to the `View`.
    pub fn add_conflict(&mut self, conflict: Conflict) {
        self.conflicts.add_entry(conflict);
    }

    /// Add multiple conflict entries to the `View`.
    pub fn add_conflicts(&mut self, conflicts: Vec<Conflict>) {
        self.conflicts.add_entries(conflicts);
    }

    /// Add a timestamp entry to the `View`.
    pub fn add_timestamp(&mut self, timestamp: Timestamp) {
        self.timestamps.add_entry(timestamp);
    }

    /// Add multiple timestamp entries to the `View`.
    pub fn add_timestamps(&mut self, timestamps: Vec<Timestamp>) {
        self.timestamps.add_entries(timestamps);
    }

    /// Get the node's opinions on a given transaction conflict.
    pub fn get_conflict_opinions(&self, id: TransactionId) -> Option<Opinions> {
        self.conflicts.get_entry_opinions(&id)
    }

    /// Get the node's opinions on a given message timestamp.
    pub fn get_timestamp_opinions(&self, id: MessageId) -> Option<Opinions> {
        self.timestamps.get_entry_opinions(&id)
    }

    /// Query a `View` for the node's opinions on a range of entry IDs.
    pub fn query(&mut self, query_ids: &QueryIds) -> Result<opinion::Opinions, Error> {
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

/// Stores the opinions of nodes across the voting pool on all voting objects.
#[derive(Default)]
pub struct Registry {
    views: RwLock<HashMap<PeerId, View>>,
}

impl Registry {
    /// Modify an existing `View` through a closure, or create a new `View` for the given node.
    pub async fn write_view(&self, node_id: PeerId, f: impl FnOnce(&mut View)) {
        let mut guard = self.views.write().await;

        if !guard.contains_key(&node_id) {
            guard.insert(
                node_id,
                View {
                    node_id: node_id,
                    conflicts: EntryMap::new(),
                    timestamps: EntryMap::new(),
                },
            );
        }

        f(guard.get_mut(&node_id).unwrap());
    }

    /// Pass a shared reference to a `View` to a closure, given a node ID.
    /// If this node cannot be found, return an error.
    pub async fn read_view(&self, node_id: PeerId, f: impl FnOnce(&View)) -> Result<(), Error> {
        let guard = self.views.read().await;
        let view = guard.get(&node_id).ok_or(Error::NodeNotFound(node_id))?;
        f(view);
        Ok(())
    }

    /// Prune the `Registry`, removing all entries created before the given duration away from the current time.
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
