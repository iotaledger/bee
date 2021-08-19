// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::mqtt::{broker::MqttBroker, handlers::HandlerError},
    storage::StorageBackend,
};

use bee_ledger::types::CreatedOutput;
use bee_message::output::OutputId;
use bee_runtime::node::Node;
use bee_storage::access::Fetch;

use bee_common::packable::Packable;
use bee_message::payload::transaction::TransactionId;
use bee_tangle::MsTangle;
use librumqttd::{LinkRx, LinkTx};
use log::{debug, warn};

pub(crate) fn spawn<N>(
    node: &mut N,
    mut transactions_included_message_tx: LinkTx,
    mut transactions_included_message_rx: LinkRx,
) where
    N: Node,
    N::Backend: StorageBackend,
{
    let storage = node.storage();
    let tangle = node.resource::<MsTangle<N::Backend>>();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT 'transactions/[transactionId]/included-message' topic handler running.");

        match transactions_included_message_rx.recv() {
            Ok(req) => {
                if let Some(data) = req {
                    match get_transaction_id_param(&data.topic) {
                        Ok(transaction_id) => {
                            // the combination of transaction id and output index `0` is valid, therefore it's safe to
                            // unwrap
                            let output_id = OutputId::new(transaction_id, 0).unwrap();

                            match Fetch::<OutputId, CreatedOutput>::fetch(&*storage, &output_id) {
                                Ok(result) => match result {
                                    Some(output) => match tangle.get(&*output.message_id()).await.map(|m| (*m).clone())
                                    {
                                        Some(message) => {
                                            if let Err(e) = transactions_included_message_tx.publish(
                                                format!("transactions/{}/included-message", transaction_id),
                                                false,
                                                message.pack_new(),
                                            ) {
                                                warn!("publishing MQTT message failed: {:?}", e);
                                            }
                                        }
                                        None => warn!("message with id {} not found", *output.message_id()),
                                    },
                                    None => warn!("output with id {} not found", output_id),
                                },
                                Err(e) => panic!("can not fetch from storage: {}", e),
                            }
                        }
                        Err(_) => warn!("invalid transaction id provided: {}", data.topic),
                    }
                } else {
                    warn!("request from client failed: no data provided")
                }
            }
            Err(e) => warn!("request from client failed: {}.", e),
        }

        debug!("MQTT 'transactions/[transactionId]/included-message' topic handler stopped.");
    });
}

fn get_transaction_id_param(topic: &String) -> Result<TransactionId, HandlerError> {
    let topic = topic.split("/").collect::<Vec<&str>>();
    if topic.len() == 2 {
        Ok(topic
            .get(1)
            .unwrap()
            .parse::<TransactionId>()
            .map_err(|_| HandlerError::InvalidParameterProvided)?)
    } else {
        Err(HandlerError::InvalidParameterProvided)
    }
}
