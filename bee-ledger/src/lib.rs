// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//#![warn(missing_docs)]

mod error;
pub mod event;
pub mod index;
mod merkle_hasher;
mod metadata;
pub mod output;
pub mod spent;
pub mod storage;
pub mod unspent;
mod white_flag;
mod worker;

pub use error::Error;
use storage::Backend;
use worker::LedgerWorker;

use bee_common::node::{Node, NodeBuilder};
use bee_protocol::MilestoneIndex;

pub fn init<N: Node>(index: u32, node_builder: N::Builder) -> N::Builder
where
    N::Backend: Backend,
{
    node_builder.with_worker_cfg::<LedgerWorker>(MilestoneIndex(index))
}
