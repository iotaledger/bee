// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::mqtt::broker::MqttBroker, storage::StorageBackend};

use bee_ledger::types::CreatedOutput;
use bee_message::output::OutputId;
use bee_runtime::node::Node;
use bee_storage::access::Fetch;

use bee_common::packable::Packable;
use bee_message::payload::transaction::TransactionId;
use bee_tangle::MsTangle;
use librumqttd::{LinkRx, LinkTx};
use log::{debug, error, warn};

pub(crate) fn spawn<N>(
    node: &mut N,
    mut transactions_included_message_tx: LinkTx,
    mut transactions_included_message_rx: LinkRx,
) where
    N: Node,
    N::Backend: StorageBackend,
{
    let tangle = node.resource::<MsTangle<N::Backend>>();
    let storage = node.storage();

    node.spawn::<MqttBroker, _, _>(|shutdown| async move {
        debug!("MQTT 'outputs/{{outputId}}' topic handler running.");

        match transactions_included_message_rx.recv() {
            Ok(req) => {
                if let Some(data) = req {
                    let topic = data.topic.split("/").collect::<Vec<&str>>();
                    if topic.len() == 2 {
                        let transaction_id = match topic.get(1).unwrap().parse::<TransactionId>() {
                            Ok(transaction_id) => transaction_id,
                            Err(_) => return,
                        };
                        let output_id = OutputId::new(transaction_id, 0).unwrap();

                        match Fetch::<OutputId, CreatedOutput>::fetch(&*storage, &output_id) {
                            Ok(result) => match result {
                                Some(output) => match tangle.get(&*output.message_id()).await.map(|m| (*m).clone()) {
                                    Some(message) => {
                                        if let Err(e) = transactions_included_message_tx.publish(
                                            format!("outputs/{}", output_id),
                                            false,
                                            message.pack_new(),
                                        ) {
                                            warn!("Publishing MQTT message failed. Cause: {:?}", e);
                                        }
                                    }
                                    None => return,
                                },
                                None => return,
                            },
                            Err(e) => panic!("can not fetch from storage: {}", e),
                        }
                    }
                }
            }
            Err(e) => error!("response from consensus worker failed: {}.", e),
        }

        debug!("MQTT 'outputs/{{outputId}}' topic handler stopped.");
    });
}
