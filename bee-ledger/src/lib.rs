// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//#![warn(missing_docs)]

pub mod error;
pub mod event;
pub mod model;
pub mod storage;

mod merkle_hasher;
mod metadata;
mod white_flag;
mod worker;

pub use storage::StorageBackend;
pub use worker::{LedgerWorker, LedgerWorkerEvent};

use bee_common_pt2::node::{Node, NodeBuilder};

pub const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

pub fn init<N>(node_builder: N::Builder) -> N::Builder
where
    N: Node,
    N::Backend: StorageBackend,
{
    node_builder.with_worker::<LedgerWorker>()
}
