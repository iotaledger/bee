// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    opinion::{Opinion, Opinions, QueryIds},
    Error,
};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    payload::transaction::{TransactionId, TRANSACTION_ID_LENGTH},
    MessageId, MESSAGE_ID_LENGTH,
};

use tokio::sync::RwLock;

use core::str::FromStr;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Holds a conflicting transaction ID and its opinion.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Conflict {
    /// Conflicting transaction ID.
    pub id: TransactionId,
    /// Opinion of the conflict.
    pub opinion: Opinion,
}

impl Packable for Conflict {
    type Error = Error;

    fn packed_len(&self) -> usize {
        TRANSACTION_ID_LENGTH + 0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.id.pack(writer)?;
        self.opinion.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let transaction_id = TransactionId::unpack(reader)?;
        let opinion = Opinion::unpack(reader)?;

        Ok(Self {
            id: transaction_id,
            opinion,
        })
    }
}

/// Holds a message ID and its timestamp opinion.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Timestamp {
    /// Message ID.
    pub id: MessageId,
    /// Opinion of the message timestamp.
    pub opinion: Opinion,
}

impl Packable for Timestamp {
    type Error = Error;

    fn packed_len(&self) -> usize {
        MESSAGE_ID_LENGTH + 0u8.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.id.pack(writer)?;
        self.opinion.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let message_id = MessageId::unpack(reader)?;
        let opinion = Opinion::unpack(reader)?;

        Ok(Self {
            id: message_id,
            opinion,
        })
    }
}

#[derive(Clone)]
pub struct Entry {
    pub opinions: Opinions,
    pub timestamp: u64,
}

// TODO String -> Node ID
pub struct View {
    pub node_id: String,
    pub conflicts: RwLock<HashMap<TransactionId, Entry>>,
    pub timestamps: RwLock<HashMap<MessageId, Entry>>,
}

// TODO clean-up (generics)
impl View {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            conflicts: RwLock::new(HashMap::new()),
            timestamps: RwLock::new(HashMap::new()),
        }
    }

    pub fn id(&self) -> &str {
        &self.node_id
    }

    pub async fn add_conflict(&self, conflict: Conflict) {
        let mut conflicts_guard = self.conflicts.write().await;

        if conflicts_guard.contains_key(&conflict.id) {
            conflicts_guard.insert(
                conflict.id,
                Entry {
                    opinions: Opinions::new(vec![conflict.opinion]),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Clock may have gone backwards")
                        .as_millis() as u64,
                },
            );
        } else {
            // This will never fail.
            let entry = conflicts_guard.get_mut(&conflict.id).unwrap();
            entry.opinions.push(conflict.opinion);
        }
    }

    pub async fn add_conflicts(&self, conflicts: Vec<Conflict>) {
        let mut conflicts_guard = self.conflicts.write().await;

        for conflict in conflicts.iter() {
            if conflicts_guard.contains_key(&conflict.id) {
                conflicts_guard.insert(
                    conflict.id,
                    Entry {
                        opinions: Opinions::new(vec![conflict.opinion]),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Clock may have gone backwards")
                            .as_millis() as u64,
                    },
                );
            } else {
                // This will never fail.
                let entry = conflicts_guard.get_mut(&conflict.id).unwrap();
                entry.opinions.push(conflict.opinion);
            }
        }
    }

    pub async fn add_timestamp(&self, timestamp: Timestamp) {
        let mut timestamps_guard = self.timestamps.write().await;

        if timestamps_guard.contains_key(&timestamp.id) {
            timestamps_guard.insert(
                timestamp.id,
                Entry {
                    opinions: Opinions::new(vec![timestamp.opinion]),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Clock may have gone backwards")
                        .as_millis() as u64,
                },
            );
        } else {
            // This will never fail.
            let entry = timestamps_guard.get_mut(&timestamp.id).unwrap();
            entry.opinions.push(timestamp.opinion);
        }
    }

    pub async fn add_timestamps(&self, timestamps: Vec<Timestamp>) {
        let mut timestamps_guard = self.timestamps.write().await;

        for timestamp in timestamps.iter() {
            if timestamps_guard.contains_key(&timestamp.id) {
                timestamps_guard.insert(
                    timestamp.id,
                    Entry {
                        opinions: Opinions::new(vec![timestamp.opinion]),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Clock may have gone backwards")
                            .as_millis() as u64,
                    },
                );
            } else {
                // This will never fail.
                let entry = timestamps_guard.get_mut(&timestamp.id).unwrap();
                entry.opinions.push(timestamp.opinion);
            }
        }
    }

    pub async fn get_conflict_opinions(&self, id: TransactionId) -> Option<Opinions> {
        self.conflicts.read().await.get(&id).map(|entry| entry.opinions.clone())
    }

    pub async fn get_timestamp_opinions(&self, id: MessageId) -> Option<Opinions> {
        self.timestamps
            .read()
            .await
            .get(&id)
            .map(|entry| entry.opinions.clone())
    }

    pub async fn query(&self, query_ids: &QueryIds) -> Result<Opinions, Error> {
        // TODO default empty `Opinions`.
        let mut opinions = Opinions::new(vec![]);

        {
            let conflicts_guard = self.conflicts.read().await;
            for id in query_ids.conflict_ids.iter() {
                if let Some(conflict_opinions) = self.get_conflict_opinions(TransactionId::from_str(id)?).await {
                    if !conflict_opinions.is_empty() {
                        // This will never fail.
                        opinions.push(*conflict_opinions.last().unwrap());
                    } else {
                        opinions.push(Opinion::Unknown);
                    }
                } else {
                    opinions.push(Opinion::Unknown);
                };
            }
        }

        {
            let timestamps_guard = self.timestamps.read().await;
            for id in query_ids.timestamp_ids.iter() {
                if let Some(timestamp_opinions) = self.get_timestamp_opinions(MessageId::from_str(id)?).await {
                    if !timestamp_opinions.is_empty() {
                        // This will never fail.
                        opinions.push(*timestamp_opinions.last().unwrap());
                    } else {
                        opinions.push(Opinion::Unknown);
                    }
                } else {
                    opinions.push(Opinion::Unknown);
                };
            }
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
    pub async fn clean(&self, duration: Duration) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
            .as_millis() as u64;

        let guard = self.views.write().await;
        for (_, view) in guard.iter() {
            let filter = |entry: &Entry| -> bool { now - entry.timestamp < duration.as_millis() as u64 };

            view.conflicts.write().await.retain(|_, entry| filter(entry));
            view.timestamps.write().await.retain(|_, entry| filter(entry));
        }
    }
}
