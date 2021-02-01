// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod download;

pub(crate) mod constants;
pub(crate) mod kind;

pub mod config;
pub mod error;
pub mod event;
pub mod header;
pub mod info;
pub mod milestone_diff;
pub mod storage;
pub mod worker;

pub use error::Error;
pub use header::SnapshotHeader;
pub use info::SnapshotInfo;
pub use worker::SnapshotWorker;

use bee_runtime::node::{Node, NodeBuilder};

pub async fn init<N: Node>(config: &config::SnapshotConfig, network_id: u64, node_builder: N::Builder) -> N::Builder
where
    N::Backend: storage::StorageBackend,
{
    node_builder.with_worker_cfg::<worker::SnapshotWorker>((network_id, config.clone()))
}
