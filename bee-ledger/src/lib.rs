// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//#![warn(missing_docs)]

pub mod config;
pub mod dust;
pub mod error;
pub mod event;
pub mod state;
pub mod storage;
pub mod types;

mod white_flag;
mod worker;

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
