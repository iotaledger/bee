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

use storage::Backend;
use worker::LedgerWorker;

use bee_common_pt2::node::{Node, NodeBuilder};
use bee_protocol::MilestoneIndex;

pub fn init<N: Node>(index: u32, node_builder: N::Builder) -> N::Builder
where
    N::Backend: Backend,
{
    node_builder.with_worker_cfg::<LedgerWorker>(MilestoneIndex(index))
}
