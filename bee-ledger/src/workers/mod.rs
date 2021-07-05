// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing workers required to create and maintain the ledger state.

pub mod consensus;
pub mod event;
pub mod pruning;
pub mod snapshot;
pub mod storage;
pub mod error;

pub use storage::StorageBackend;

use consensus::ConsensusWorker;
use pruning::config::PruningConfig;
use snapshot::{config::SnapshotConfig, worker::SnapshotWorker};

use bee_runtime::node::{Node, NodeBuilder};

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
