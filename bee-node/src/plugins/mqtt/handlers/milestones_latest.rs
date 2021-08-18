// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::super::event::*;

use crate::{plugins::mqtt::handlers::spawn_static_topic_handler, storage::StorageBackend};

use bee_runtime::node::Node;
use bee_tangle::event::LatestMilestoneChanged;

use librumqttd::LinkTx;

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
            // MilestonePayload as JSON
            let ms_payload_json = serde_json::to_string(&MilestonePayload {
                index: *event.index,
                timestamp: event.milestone.timestamp(),
            })
            .expect("error serializing to json");

            ("milestones/latest", ms_payload_json)
        },
    );
}
