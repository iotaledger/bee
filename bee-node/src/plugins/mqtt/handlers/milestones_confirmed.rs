// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::handlers::spawn_static_topic_handler, storage::StorageBackend};

use bee_ledger::workers::event::MilestoneConfirmed;
use bee_runtime::node::Node;

use librumqttd::LinkTx;
use serde::Serialize;

#[derive(Serialize)]
struct MilestoneConfirmedResponse {
    pub index: u32,
    pub timestamp: u64,
}

pub(crate) fn spawn<N>(node: &mut N, milestones_confirmed_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    spawn_static_topic_handler(
        node,
        milestones_confirmed_tx,
        "milestones/confirmed",
        |event: MilestoneConfirmed| {
            let response = serde_json::to_string(&MilestoneConfirmedResponse {
                index: *event.index,
                timestamp: event.timestamp,
            })
            .expect("error serializing to json");

            ("milestones/confirmed", response)
        },
    );
}
