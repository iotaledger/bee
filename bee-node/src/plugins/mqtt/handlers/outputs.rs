// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::StorageBackend,
};

use bee_ledger::workers::event::{MilestoneConfirmed, MessageReferenced};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use bee_protocol::types::metrics::NodeMetrics;
use futures::StreamExt;
use log::{debug, error, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::plugins::mqtt::broker::MqttBroker;
use librumqttd as mqtt;
use mqtt::{LinkTx, LinkRx};
use bee_tangle::{MsTangle, ConflictReason};
use bee_message::payload::Payload;
use bee_rest_api::types::dtos::LedgerInclusionStateDto;
use bee_rest_api::types::responses::{MessageMetadataResponse, OutputResponse};
use bee_ledger::workers::consensus::{ConsensusWorkerCommand, ConsensusWorker};
use bee_ledger::types::{CreatedOutput, LedgerIndex};
use rumqttlog::Data;
use bee_message::output::OutputId;
use bee_storage::access::Fetch;
use futures::channel::oneshot;
use bee_ledger::{
    types::{ConsumedOutput},
    workers::{error::Error},
};

pub(crate) fn spawn<N>(
    node: &mut N,
    mut outputs_tx: LinkTx,
    mut outputs_rx: LinkRx,
)
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
                            Err(_) => return
                        };

                        let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>();

                        if let Err(e) = consensus_worker.send(ConsensusWorkerCommand::FetchOutput(output_id.clone(), cmd_tx)) {
                            panic!("request to consensus worker failed: {}.", e);
                        }

                        if let Ok(response) = cmd_rx.await {
                            match response {
                                (Ok(response), ledger_index) => match response {
                                    Some(output) => {


                                        let is_spent = match Fetch::<OutputId, ConsumedOutput>::fetch(&*storage, &output_id) {
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
                                    }
                                    None => panic!("output not found"),
                                }
                                (Err(e), _) => panic!("unable to fetch the output: {}", e)
                            }
                        }

                    }






                }

            }
            Err(e) => error!("response from consensus worker failed: {}.", e)
        }

        debug!("MQTT 'outputs/{{outputId}}' topic handler stopped.");
    });

}