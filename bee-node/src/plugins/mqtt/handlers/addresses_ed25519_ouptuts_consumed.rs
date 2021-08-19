// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::handlers::spawn_static_topic_handler, storage::StorageBackend};

use bee_ledger::workers::event::OutputConsumed;
use bee_message::{address::Address, output::Output};
use bee_rest_api::types::responses::OutputResponse;
use bee_runtime::node::Node;

use librumqttd::LinkTx;

pub(crate) fn spawn<N>(node: &mut N, addresses_ed25519_ouptuts_created_tx: LinkTx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    spawn_static_topic_handler(
        node,
        addresses_ed25519_ouptuts_created_tx,
        "addresses/ed25519/[address]/outputs created",
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

            let address = match event.output {
                Output::SignatureLockedSingle(o) => *o.address(),
                Output::SignatureLockedDustAllowance(o) => *o.address(),
                _ => panic!("output type not supported"),
            };

            let ed_address = match address {
                Address::Ed25519(a) => a,
            };

            (
                format!("addresses/ed25519/{}/outputs", ed_address.to_string()),
                output_response_json,
            )
        },
    );
}
