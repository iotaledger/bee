// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//#![warn(missing_docs)]

mod error;
pub mod event;
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
pub use worker::LedgerWorkerEvent;

use bee_common::{
    event::Bus,
    node::{Node, NodeBuilder},
};
use bee_protocol::{config::ProtocolCoordinatorConfig, MilestoneIndex};

use std::sync::Arc;

pub fn init<N: Node>(
    index: u32,
    coo_config: ProtocolCoordinatorConfig,
    node_builder: N::Builder,
    bus: Arc<Bus<'static>>,
) -> N::Builder
where
    N::Backend: Backend,
{
    node_builder.with_worker_cfg::<LedgerWorker>((MilestoneIndex(index), coo_config, bus.clone()))
}

pub fn events<N: Node>(_node: &N, _bus: Arc<Bus<'static>>) {
    // let ledger_worker = node.worker::<LedgerWorker>().unwrap().tx.clone();
    //
    // bus.add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
    //     if let Err(e) = ledger_worker.send(LedgerWorkerEvent::Confirm(latest_solid_milestone.0.clone())) {
    //         warn!(
    //             "Sending solid milestone {:?} to confirmation failed: {:?}.",
    //             latest_solid_milestone.0.index(),
    //             e
    //         );
    //     }
    // });
}
