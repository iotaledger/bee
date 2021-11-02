// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that contains foundational building blocks for the IOTA Tangle.

#![deny(missing_docs)]

/// Types used for tangle configuration.
pub mod config;
/// Types that represent tangle events.
pub mod event;
/// Message flags.
pub mod flags;
/// Message data, including message flags.
pub mod metadata;
/// Types used to represent SEPs (Solid Entry Points).
pub mod solid_entry_point;
/// Types used for interoperation with a node's storage layer.
pub mod storage;
/// Milestone-enabled tangle type.
pub mod tangle;
/// The overall `TangleWorker` type. Used as part of the bee runtime in a node.
pub mod tangle_worker;
/// A worker that periodically cleans the tip pool.
pub mod tip_pool_cleaner_worker;
/// Common tangle traversal functionality.
pub mod traversal;
/// Types used to represent unreferenced messages.
pub mod unreferenced_message;
/// The URTS tips pool.
pub mod urts;

mod conflict;
mod vec_set;
mod vertex;
mod vertices;

pub use conflict::ConflictReason;
pub use tangle::Tangle;
pub use tangle_worker::TangleWorker;

use tip_pool_cleaner_worker::TipPoolCleanerWorker;

use crate::vec_set::VecSet;

use bee_message::Message;
use bee_runtime::node::{Node, NodeBuilder};

use std::{ops::Deref, sync::Arc};

/// A thread-safe reference to a `Message`.
#[derive(Clone)]
pub struct MessageRef(pub(crate) Arc<Message>);

impl Deref for MessageRef {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// Initiate the tangle on top of the given node builder.
pub fn init<N: Node>(tangle_config: &config::TangleConfig, node_builder: N::Builder) -> N::Builder
where
    N::Backend: storage::StorageBackend,
{
    node_builder
        .with_worker_cfg::<TangleWorker>(tangle_config.clone())
        .with_worker::<TipPoolCleanerWorker>()
}
