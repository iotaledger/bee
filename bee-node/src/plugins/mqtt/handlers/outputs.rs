// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::broker::MqttBroker, storage::StorageBackend};

use bee_ledger::{
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
    workers::{
        consensus::{ConsensusWorker, ConsensusWorkerCommand},
        error::Error,
    },
};
use bee_message::output::OutputId;
use bee_rest_api::types::responses::OutputResponse;
use bee_runtime::node::Node;
use bee_storage::access::Fetch;

use futures::channel::oneshot;
use librumqttd::{LinkRx, LinkTx};
use log::{debug, error, warn};

pub(crate) fn spawn<N>(node: &mut N, mut outputs_tx: LinkTx, mut outputs_rx: LinkRx)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let storage = node.storage();
    let consensus_worker = node.worker::<ConsensusWorker>().unwrap().tx.clone();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT 'outputs/{{outputId}}' topic handler running.");

        match outputs_rx.recv() {
            Ok(req) => {
                if let Some(data) = req {
                    let topic = data.topic.split("/").collect::<Vec<&str>>();
                    if topic.len() == 2 {
                        let output_id = match topic.get(1).unwrap().parse::<OutputId>() {
                            Ok(output_id) => output_id,
                            Err(_) => return,
                        };

                        let (cmd_tx, cmd_rx) =
                            oneshot::channel::<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>();

                        if let Err(e) =
                            consensus_worker.send(ConsensusWorkerCommand::FetchOutput(output_id.clone(), cmd_tx))
                        {
                            panic!("request to consensus worker failed: {}.", e);
                        }

                        if let Ok(response) = cmd_rx.await {
                            match response {
                                (Ok(response), ledger_index) => match response {
                                    Some(output) => {
                                        let is_spent =
                                            match Fetch::<OutputId, ConsumedOutput>::fetch(&*storage, &output_id) {
                                                Ok(is_spent) => is_spent,
                                                Err(e) => panic!("unable to fetch the output: {}", e),
                                            };

                                        let output_response_json = serde_json::to_string(&OutputResponse {
                                            message_id: output.message_id().to_string(),
                                            transaction_id: output_id.transaction_id().to_string(),
                                            output_index: output_id.index(),
                                            is_spent: is_spent.is_some(),
                                            output: output.inner().into(),
                                            ledger_index: *ledger_index,
                                        })
                                        .expect("error serializing to json");

                                        if let Err(e) = outputs_tx.publish(
                                            format!("outputs/{}", output_id),
                                            false,
                                            output_response_json,
                                        ) {
                                            warn!("Publishing MQTT message failed. Cause: {:?}", e);
                                        }
                                    }
                                    None => panic!("output not found"),
                                },
                                (Err(e), _) => panic!("unable to fetch the output: {}", e),
                            }
                        }
                    }
                }
            }
            Err(e) => error!("response from consensus worker failed: {}.", e),
        }

        debug!("MQTT 'outputs/{{outputId}}' topic handler stopped.");
    });
}
