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
    messages_indexation_tx: LinkTx,
) where
    N: Node,
    N::Backend: StorageBackend,
{
    spawn_static_topic_handler(
        node,
        messages_indexation_tx,
        "messages/indexation/{index}",
        |event: NewIndexationMessage| {
            (format!("messages/indexation/{}", event.index.to_string()), event.message.pack_new())
        },
    );
}