// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that contains foundational building blocks for the IOTA Tangle.

// #![warn(missing_docs)]

pub mod config;
pub mod flags;
pub mod metadata;
pub mod ms_tangle;
pub mod storage;
pub mod traversal;
pub mod urts;
// pub mod worker;

mod tangle;
mod vertex;
mod shashmap;

pub use ms_tangle::MsTangle;
pub use tangle::{Hooks, Tangle};

use bee_message::Message;

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

// pub fn init<N: Node>(node_builder: N::Builder) -> N::Builder
// where
//     N::Backend: StorageBackend,
// {
//     node_builder.with_worker::<TangleWorker>()
// }
