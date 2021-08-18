// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::handlers::spawn_static_topic_handler, storage::StorageBackend};

use bee_ledger::workers::event::OutputConsumed;
use bee_rest_api::types::responses::OutputResponse;
use bee_runtime::node::Node;

use librumqttd::LinkTx;

pub(crate) fn spawn<N>(node: &mut N, outputs_created_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    spawn_static_topic_handler(
        node,
        outputs_created_tx,
        "outputs/{outputId} created",
        |event: OutputConsumed| {
            let output_response_json = serde_json::to_string(&OutputResponse {
                message_id: event.message_id.to_string(),
                transaction_id: event.output_id.transaction_id().to_string(),
                output_index: event.output_id.index(),
                is_spent: false,
                output: (&event.output).into(),
                ledger_index: 0, // TODO: set actual ledger-index
            })
            .expect("error serializing to json");

            (format!("outputs/{}", event.output_id.to_string()), output_response_json)
        },
    );
}
