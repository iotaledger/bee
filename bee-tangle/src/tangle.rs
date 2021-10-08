// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{metadata::MessageMetadata, storage::StorageBackend, vertex::Vertex, MessageRef};

use bee_message::{Message, MessageId};
use bee_runtime::resource::ResourceHandle;

use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use log::{info, trace};
use lru::LruCache;
use tokio::sync::{Mutex, RwLock, RwLockWriteGuard};

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

pub const DEFAULT_CACHE_LEN: usize = 100_000;
const CACHE_THRESHOLD_FACTOR: f64 = 0.1;

/// A foundational, thread-safe graph datastructure to represent the IOTA Tangle.
pub struct Tangle<B> {
    vertices: RwLock<HashMap<MessageId, Vertex>>,

    cache_queue: Mutex<LruCache<MessageId, (), DefaultHashBuilder>>,
    max_len: AtomicUsize,

    storage: ResourceHandle<B>,
}

#[allow(clippy::len_without_is_empty)]
impl<B: StorageBackend> Tangle<B> {
    /// Creates a new Tangle.
    pub fn new(storage: ResourceHandle<B>) -> Self {
        Self {
            vertices: RwLock::new(HashMap::new()),

            cache_queue: Mutex::new(LruCache::unbounded_with_hasher(DefaultHashBuilder::default())),
            max_len: AtomicUsize::new(DEFAULT_CACHE_LEN),

            storage,
        }
    }

    /// Create a new tangle with the given capacity.
    pub fn with_capacity(self, cap: usize) -> Self {
        Self {
            cache_queue: Mutex::new(LruCache::with_hasher(cap + 1, DefaultHashBuilder::default())),
            ..self
        }
    }

    /// Change the maximum number of entries to store in the cache.
    pub fn resize(&self, len: usize) {
        self.max_len.store(len, Ordering::Relaxed);
    }

    /// Return a reference to the storage  used by this tangle.
    pub fn storage(&self) -> &B {
        &self.storage
    }

    async fn insert_inner(
        &self,
        message_id: MessageId,
        message: Message,
        metadata: MessageMetadata,
        prevent_eviction: bool,
    ) -> Option<MessageRef> {
        let mut vertices = self.vertices.write().await;
        let vertex = vertices.entry(message_id).or_insert_with(Vertex::empty);

        if prevent_eviction {
            vertex.prevent_eviction();
        }

        let msg = if vertex.message().is_some() {
            None
        } else {
            let parents = message.parents().clone();

            vertex.insert_message_and_metadata(message, metadata);
            let msg = vertex.message().cloned();

            let mut cache_queue = self.cache_queue.lock().await;

            // Insert children for parents
            for &parent in parents.iter() {
                let children = vertices.entry(parent).or_insert_with(Vertex::empty);
                children.add_child(message_id);

                // Insert cache queue entry to track eviction priority
                cache_queue.put(parent, ());
            }

            // Insert cache queue entry to track eviction priority
            cache_queue.put(message_id, ());

            msg
        };

        drop(vertices);

        self.perform_eviction().await;

        msg
    }

    /// Inserts a message, and returns a thread-safe reference to it in case it didn't already exist.
    pub async fn insert(
        &self,
        message_id: MessageId,
        message: Message,
        metadata: MessageMetadata,
    ) -> Option<MessageRef> {
        let exists = self.pull_message(&message_id, true).await;

        let msg = self.insert_inner(message_id, message.clone(), metadata, !exists).await;

        self.vertices
            .write()
            .await
            .get_mut(&message_id)
            .expect("Just-inserted message is missing")
            .allow_eviction();

        if msg.is_some() {
            // Write parents to DB
            for &parent in message.parents().iter() {
                self.storage_insert_approver(parent, message_id)
                    .unwrap_or_else(|e| info!("Failed to update approvers for message {:?}", e));
            }

            // Insert into backend using hooks
            self.storage_insert(message_id, message, metadata)
                .unwrap_or_else(|e| info!("Failed to insert message {:?}", e));
        }

        msg
    }

    async fn get_inner(&self, message_id: &MessageId) -> Option<impl DerefMut<Target = Vertex> + '_> {
        let res = RwLockWriteGuard::try_map(self.vertices.write().await, |m| m.get_mut(message_id)).ok();

        if res.is_some() {
            // Update message_id priority
            self.cache_queue.lock().await.put(*message_id, ());
        }

        res
    }

    /// Get the data of a vertex associated with the given `message_id`.
    pub async fn get_with<R>(&self, message_id: &MessageId, f: impl FnOnce(&mut Vertex) -> R) -> Option<R> {
        let exists = self.pull_message(message_id, true).await;

        self.get_inner(message_id).await.map(|mut v| {
            if exists {
                v.allow_eviction();
            }
            f(&mut v)
        })
    }

    /// Get the data of a vertex associated with the given `message_id`.
    pub async fn get(&self, message_id: &MessageId) -> Option<MessageRef> {
        self.get_with(message_id, |v| v.message().cloned()).await.flatten()
    }

    async fn contains_inner(&self, message_id: &MessageId) -> bool {
        self.vertices
            .read()
            .await
            .get(message_id)
            .map_or(false, |v| v.message().is_some())
    }

    /// Returns whether the message is stored in the Tangle.
    pub async fn contains(&self, message_id: &MessageId) -> bool {
        self.contains_inner(message_id).await || self.pull_message(message_id, false).await
    }

    /// Get the metadata of a vertex associated with the given `message_id`.
    pub async fn get_metadata(&self, message_id: &MessageId) -> Option<MessageMetadata> {
        self.get_with(message_id, |v| v.metadata().cloned()).await.flatten()
    }

    /// Get the metadata of a vertex associated with the given `message_id`.
    pub async fn get_vertex(&self, message_id: &MessageId) -> Option<impl Deref<Target = Vertex> + '_> {
        let exists = self.pull_message(message_id, true).await;

        self.get_inner(message_id).await.map(|mut v| {
            if exists {
                v.allow_eviction();
            }
            v
        })
    }

    /// Updates the metadata of a vertex.
    pub async fn update_metadata<R, Update>(&self, message_id: &MessageId, update: Update) -> Option<R>
    where
        Update: FnOnce(&mut MessageMetadata) -> R,
    {
        let exists = self.pull_message(message_id, true).await;
        let mut vertices = self.vertices.write().await;
        if let Some(vertex) = vertices.get_mut(message_id) {
            // If we previously blocked eviction, allow it again
            if exists {
                vertex.allow_eviction();
            }

            let r = vertex.metadata_mut().map(|m| update(m));

            if let Some((msg, meta)) = vertex.message_and_metadata() {
                let (msg, meta) = ((&**msg).clone(), *meta);

                // Insert cache queue entry to track eviction priority
                self.cache_queue.lock().await.put(*message_id, ());

                drop(vertices);

                self.storage_insert(*message_id, msg, meta)
                    .unwrap_or_else(|e| info!("Failed to update metadata for message {:?}", e));
            }

            r
        } else {
            None
        }
    }

    async fn children_inner(&self, message_id: &MessageId) -> Option<impl Deref<Target = Vec<MessageId>> + '_> {
        struct Wrapper<'a> {
            children: Vec<MessageId>,
            phantom: PhantomData<&'a ()>,
        }

        impl<'a> Deref for Wrapper<'a> {
            type Target = Vec<MessageId>;

            fn deref(&self) -> &Self::Target {
                &self.children
            }
        }

        let vertices = self.vertices.read().await;
        let v = vertices
            .get(message_id)
            // Skip approver lists that are not exhaustive
            .filter(|v| v.children_exhaustive());

        let children = match v {
            Some(v) => {
                // Insert cache queue entry to track eviction priority
                self.cache_queue.lock().await.put(*message_id, ());
                let children = v.children().to_vec();
                drop(vertices);
                children
            }
            None => {
                // Insert cache queue entry to track eviction priority
                self.cache_queue.lock().await.put(*message_id, ());
                drop(vertices);
                let to_insert = match self.storage_fetch_approvers(message_id) {
                    Err(e) => {
                        info!("Failed to update approvers for message message {:?}", e);
                        Vec::new()
                    }
                    Ok(None) => Vec::new(),
                    Ok(Some(approvers)) => approvers,
                };

                let mut vertices = self.vertices.write().await;
                let v = vertices.entry(*message_id).or_insert_with(Vertex::empty);

                // We've just fetched approvers from the database, so we have all the information available to us now.
                // Therefore, the approvers list is exhaustive (i.e: it contains all knowledge we have).
                v.set_exhaustive();

                for child in to_insert {
                    v.add_child(child);
                }

                v.children().to_vec()
            }
        };

        Some(Wrapper {
            children,
            phantom: PhantomData,
        })
    }

    /// Returns the children of a vertex, if we know about them.
    pub async fn get_children(&self, message_id: &MessageId) -> Option<Vec<MessageId>> {
        // Effectively atomic
        self.children_inner(message_id).await.map(|approvers| approvers.clone())
    }

    #[cfg(test)]
    pub async fn clear(&mut self) {
        self.vertices.write().await.clear();
    }

    // Attempts to pull the message from the storage, returns true if successful.
    async fn pull_message(&self, message_id: &MessageId, prevent_eviction: bool) -> bool {
        let contains_now = if prevent_eviction {
            self.vertices.write().await.get_mut(message_id).map_or(false, |v| {
                if v.message().is_some() {
                    v.prevent_eviction();
                    true
                } else {
                    false
                }
            })
        } else {
            self.contains_inner(message_id).await
        };

        // If the tangle already contains the message, do no more work
        if contains_now {
            // Insert cache queue entry to track eviction priority
            self.cache_queue.lock().await.put(*message_id, ());

            true
        } else if let Ok(Some((msg, metadata))) = self.storage_get(message_id) {
            // Insert cache queue entry to track eviction priority
            self.cache_queue.lock().await.put(*message_id, ());

            self.insert_inner(*message_id, msg, metadata, prevent_eviction).await;

            true
        } else {
            false
        }
    }

    async fn perform_eviction(&self) {
        let max_len = self.max_len.load(Ordering::Relaxed);
        let len = self.vertices.read().await.len();
        if len > max_len {
            let mut vertices = self.vertices.write().await;
            let mut cache_queue = self.cache_queue.lock().await;
            while vertices.len() > ((1.0 - CACHE_THRESHOLD_FACTOR) * max_len as f64) as usize {
                let remove = cache_queue.pop_lru().map(|(id, _)| id);

                if let Some(message_id) = remove {
                    if let Some(v) = vertices.remove(&message_id) {
                        if !v.can_evict() {
                            // Reinsert it if we're not permitted to evict it yet (because something is using it)
                            vertices.insert(message_id, v);
                            cache_queue.put(message_id, ());
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }
}

impl<B: StorageBackend> Tangle<B> {
    fn storage_get(&self, id: &MessageId) -> Result<Option<(Message, MessageMetadata)>, B::Error> {
        trace!("Attempted to fetch message {:?}", id);
        let msg = self.storage.fetch(id)?;
        let meta = self.storage.fetch(id)?;
        Ok(msg.zip(meta))
    }

    fn storage_insert(&self, id: MessageId, tx: Message, metadata: MessageMetadata) -> Result<(), B::Error> {
        trace!("Attempted to insert message {:?}", id);
        self.storage.insert(&id, &tx)?;
        self.storage.insert(&id, &metadata)?;
        Ok(())
    }

    fn storage_fetch_approvers(&self, id: &MessageId) -> Result<Option<Vec<MessageId>>, B::Error> {
        trace!("Attempted to fetch approvers for message {:?}", id);
        self.storage.fetch(id)
    }

    fn storage_insert_approver(&self, id: MessageId, approver: MessageId) -> Result<(), B::Error> {
        trace!("Attempted to insert approver for message {:?}", id);
        self.storage.insert(&(id, approver), &())
    }
}
