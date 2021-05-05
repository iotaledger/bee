// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod consensus;
pub mod error;
pub mod event;
pub mod pruning;
pub mod snapshot;
pub mod storage;

pub use storage::StorageBackend;

use bee_runtime::node::{Node, NodeBuilder};

use consensus::ConsensusWorker;
use pruning::config::PruningConfig;
use snapshot::config::SnapshotConfig;

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
    node_builder.with_worker_cfg::<ConsensusWorker>((network_id, snapshot_config, pruning_config))
}
