// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//#![warn(missing_docs)]

pub mod balance;
pub mod conflict;
pub mod dust;
pub mod error;
pub mod event;
pub mod model;
pub mod state;
pub mod storage;

mod merkle_hasher;
mod metadata;
mod white_flag;
mod worker;

pub use conflict::ConflictReason;
pub use storage::StorageBackend;
pub use worker::{LedgerWorker, LedgerWorkerEvent};

use bee_runtime::node::{Node, NodeBuilder};

pub fn init<N>(node_builder: N::Builder) -> N::Builder
where
    N: Node,
    N::Backend: StorageBackend,
{
    node_builder.with_worker::<LedgerWorker>()
}
