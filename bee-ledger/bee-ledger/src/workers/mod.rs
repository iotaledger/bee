// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing workers required to create and maintain the ledger state.

pub mod consensus;
pub mod error;
pub mod event;
pub mod pruning;
pub mod snapshot;
pub mod storage;

use bee_runtime::node::{Node, NodeBuilder};

pub use self::storage::StorageBackend;
use self::{
    consensus::ConsensusWorker,
    pruning::config::PruningConfig,
    snapshot::{config::SnapshotConfig, worker::SnapshotWorker},
};

/// Initializes the ledger workers.
pub fn init<N>(
    node_builder: N::Builder,
    network_id: u64,
    snapshot_config: SnapshotConfig,
    pruning_config: PruningConfig,
) -> N::Builder
where
    N: Node,
    N::Backend: StorageBackend,
{
    node_builder
        .with_worker_cfg::<SnapshotWorker>((network_id, snapshot_config.clone()))
        .with_worker_cfg::<ConsensusWorker>((snapshot_config, pruning_config))
}
