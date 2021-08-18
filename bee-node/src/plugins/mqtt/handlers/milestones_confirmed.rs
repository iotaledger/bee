// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::super::event::*;

use crate::{plugins::mqtt::handlers::spawn_static_topic_handler, storage::StorageBackend};

use bee_runtime::node::Node;
use bee_tangle::event::ConfirmedMilestoneChanged;

use librumqttd::LinkTx;

pub(crate) fn spawn<N>(node: &mut N, milestones_confirmed_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    spawn_static_topic_handler(
        node,
        milestones_confirmed_tx,
        "milestones/confirmed",
        |event: ConfirmedMilestoneChanged| {
            // MilestonePayload as JSON
            let ms_payload_json = serde_json::to_string(&MilestonePayload {
                index: *event.index,
                timestamp: event.milestone.timestamp(),
            })
            .expect("error serializing to json");

            ("milestones/confirmed", ms_payload_json)
        },
    );
}
