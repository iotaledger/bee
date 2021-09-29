// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::handlers::spawn_static_topic_handler, storage::StorageBackend};

use bee_runtime::node::Node;
use bee_tangle::event::LatestMilestoneChanged;

use librumqttd::LinkTx;
use serde::Serialize;

#[derive(Serialize)]
struct MilestoneLatestResponse {
    pub index: u32,
    pub timestamp: u64,
}

pub(crate) fn spawn<N>(node: &mut N, milestones_latest_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    spawn_static_topic_handler(
        node,
        milestones_latest_tx,
        "milestones/latest",
        |event: LatestMilestoneChanged| {
            let response = serde_json::to_string(&MilestoneLatestResponse {
                index: *event.index,
                timestamp: event.milestone.timestamp(),
            })
            .expect("error serializing to json");

            ("milestones/latest", response)
        },
    );
}
