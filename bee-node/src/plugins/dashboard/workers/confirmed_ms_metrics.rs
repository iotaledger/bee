// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    plugins::dashboard::{
        broadcast,
        websocket::{responses::confirmed_milestone_metrics, WsUsers},
        Dashboard,
    },
    storage::StorageBackend,
};

use bee_ledger::consensus::event::MilestoneConfirmed;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use bee_protocol::types::metrics::NodeMetrics;
use futures::StreamExt;
use log::{debug, error, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub(crate) fn confirmed_ms_metrics_worker<N>(node: &mut N, users: &WsUsers)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let metrics = node.resource::<NodeMetrics>();
    let bus = node.bus();
    let users = users.clone();
    let (tx, rx) = mpsc::unbounded_channel::<MilestoneConfirmed>();

    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws ConfirmedMilestoneMetrics topic handler running.");

        let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

        let mut prev_event: Option<MilestoneConfirmed> = None;
        let mut prev_new_message_count = 0;

        while let Some(event) = receiver.next().await {
            if prev_event.is_some() {

                // unwrap is safe since of the condition above
                let time_diff = event.timestamp - prev_event.unwrap().timestamp;

                let new_msg_count = metrics.new_messages();
                let new_msg_diff = new_msg_count - prev_new_message_count;
                prev_new_message_count = new_msg_count;

                let mut referenced_rate = 0.0;
                if new_msg_diff > 0 {
                    referenced_rate = (event.referenced_messages as f64 / new_msg_diff as f64) * 100.0;
                }

                // to avoid division by zero in case two milestones do have the same timestamp
                if time_diff > 0 {
                    let metrics = ConfirmedMilestoneMetrics {
                        ms_index: *event.index,
                        mps: new_msg_diff / time_diff,
                        cmps: event.referenced_messages as u64 / time_diff,
                        referenced_rate,
                        time_since_last_ms: time_diff,
                    };
                    broadcast(confirmed_milestone_metrics::forward(metrics), &users).await;
                }  else {
                    error!("Can not calculate milestone confirmation metrics since the time difference between milestone {} and milestone {} is zero.", *event.index - 1, *event.index)
                }

            }

            prev_event = Some(event);

        }

        debug!("Ws ConfirmedMilestoneMetrics topic handler stopped.");
    });

    bus.add_listener::<Dashboard, _, _>(move |event: &MilestoneConfirmed| {
        if tx.send((*event).clone()).is_err() {
            warn!("Sending event to `confirmed_ms_metrics_worker` failed.");
        }
    });
}

#[derive(Clone)]
pub struct ConfirmedMilestoneMetrics {
    pub ms_index: u32,
    pub mps: u64,
    pub cmps: u64,
    pub referenced_rate: f64,
    pub time_since_last_ms: u64,
}
