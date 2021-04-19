// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that contains foundational building blocks for the IOTA Tangle.

#![deny(missing_docs)]

/// Types used for tangle configuration.
pub mod config;
/// Types that represent milestone events.
pub mod event;
/// Milestone message flags.
pub mod flags;
/// Milestone message data, including message flags.
pub mod metadata;
/// Milestone-enabled tangle type.
pub mod ms_tangle;
/// Types used to represent SEPs (Solid Entry Points).
pub mod solid_entry_point;
/// Types used for interoperation with a node's storage layer.
pub mod storage;
/// Common tangle traversal functionality.
pub mod traversal;
/// Types used to represent unconfirmed (unsolid) messages.
pub mod unconfirmed_message;
/// The URTS tips pool.
pub mod urts;
/// The overall `TangleWorker` type. Used as part of the bee runtime in a node.
pub mod worker;

mod vec_set;
mod tangle;
mod vertex;

pub use ms_tangle::MsTangle;
pub use tangle::{Hooks, Tangle};
pub use worker::TangleWorker;

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
    node_builder.with_worker_cfg::<TangleWorker>(tangle_config.clone())
}
