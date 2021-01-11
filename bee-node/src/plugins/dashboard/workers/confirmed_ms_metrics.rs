// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{plugins::Dashboard, storage::StorageBackend};

use bee_ledger::event::MilestoneConfirmed;
use bee_protocol::event::MpsMetricsUpdated;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream};

use futures::StreamExt;
use log::{debug, error, warn};
use tokio::sync::mpsc;

pub(crate) fn confirmed_ms_metrics_worker<N>(node: &mut N)
where
    N: Node,
    N::Backend: StorageBackend,
{
    let bus = node.bus();
    let (tx, rx) = mpsc::unbounded_channel();

    let bus_clone = bus.clone();
    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        debug!("Ws `confirmed_ms_metrics_worker` running.");

        let mut receiver = ShutdownStream::new(shutdown, rx);

        let mut prev_ms_timestamp = None;
        let mut latest_num_message_new = None;

        while let Some(event) = receiver.next().await {

            match event {

                IncomingEvent::MilestoneConfirmed(event) => {
                    if prev_ms_timestamp.is_some() && latest_num_message_new.is_some() {
                        let time_diff = event.timestamp - prev_ms_timestamp.unwrap();
                        // to avoid division by zero in case two milestones do have the same timestamp
                        if time_diff > 0 {
                            bus_clone.dispatch(ConfirmedMilestoneMetrics {
                                ms_index: *event.index,
                                mps: latest_num_message_new.unwrap() / time_diff,
                                cmps: event.referenced_messages as u64 / time_diff,
                                referenced_rate: event.referenced_messages as u64,
                                time_since_last_ms: time_diff,
                            });
                        }  else {
                            error!("Can not calculate milestone confirmation metrics since the time difference between milestone {} and milestone {} is zero.", *event.index - 1, *event.index)
                        }
                    }
                    prev_ms_timestamp = Some(event.timestamp);
                }

                IncomingEvent::MpsMetricsUpdated(event) => latest_num_message_new = Some(event.new),
            }
        }

        debug!("Ws `confirmed_ms_metrics_worker` stopped.");
    });

    {
        let tx = tx.clone();
        bus.add_listener::<Dashboard, _, _>(move |event: &MilestoneConfirmed| {
            if tx.send(IncomingEvent::MilestoneConfirmed((*event).clone())).is_err() {
                warn!("Sending event to `confirmed_ms_metrics_worker` failed.");
            }
        });
    }

    {
        let tx = tx.clone();
        bus.add_listener::<Dashboard, _, _>(move |event: &MpsMetricsUpdated| {
            if tx.send(IncomingEvent::MpsMetricsUpdated((*event).clone())).is_err() {
                warn!("Sending event to `confirmed_ms_metrics_worker` failed.");
            }
        });
    }
}

enum IncomingEvent {
    MilestoneConfirmed(MilestoneConfirmed),
    MpsMetricsUpdated(MpsMetricsUpdated),
}

#[derive(Clone)]
pub struct ConfirmedMilestoneMetrics {
    pub ms_index: u32,
    pub mps: u64,
    pub cmps: u64,
    pub referenced_rate: u64,
    pub time_since_last_ms: u64,
}
