// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::super::event::*;

use crate::storage::StorageBackend;

use bee_protocol::workers::event::{MessageProcessed, NewIndexationMessage};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::event::{ConfirmedMilestoneChanged, LatestMilestoneChanged};
use bee_common::packable::Packable;

use async_trait::async_trait;
use librumqttd as mqtt;
use log::*;
use mqtt::LinkTx;
// use rumqttlog::Data as Message;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};


use std::{
    any::{Any},
    convert::Infallible,
};

use bee_rest_api::types::responses::OutputResponse;
use bee_ledger::workers::event::{OutputCreated, OutputConsumed, MessageReferenced};
use chrono::format::format;
use bee_message::output::Output;
use bee_message::address::Address;
use crate::config::NodeConfig;
use crate::plugins::mqtt::handlers::spawn_static_topic_handler;

pub(crate) fn spawn<N>(
    node: &mut N,
    addresses_ouptuts_created_tx: LinkTx
) where
    N: Node,
    N::Backend: StorageBackend,
{
    let node_config = node.resource::<NodeConfig<N::Backend>>();
    spawn_static_topic_handler(
        node,
        addresses_ouptuts_created_tx,
        "addresses/{address}/outputs created",
        move |event: OutputConsumed| {

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
                _ => panic!("output type not supported")
            };

            (format!("addresses/{}/outputs", address.to_bech32(&(node_config.bech32_hrp))), output_response_json)
        },
    );
}