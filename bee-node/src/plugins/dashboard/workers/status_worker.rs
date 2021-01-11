// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::Dashboard;

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::node::Node;
use bee_ledger::event::MilestoneConfirmed;
use bee_protocol::event::MpsMetricsUpdated;

use futures::StreamExt;
use log::{error, info, warn};
use tokio::sync::mpsc;

pub(crate) fn init_status_worker<N>(node: &mut N)
where
    N: Node,
{
    let bus = node.bus();
    let (tx, rx) = mpsc::unbounded_channel();

    let bus_clone = bus.clone();
    node.spawn::<Dashboard, _, _>(|shutdown| async move {
        info!("Ws `status_worker` running.");

        let mut receiver = ShutdownStream::new(shutdown, rx);

        while let Some(event) = receiver.next().await {}

        info!("Ws `status_worker` stopped.");
    });

    {
        let tx = tx.clone();
        bus.add_listener::<Dashboard, _, _>(move |event: &MilestoneConfirmed| {
            if tx.send(IncomingEvent::MilestoneConfirmed((*event).clone())).is_err() {
                warn!("Sending event to `status_worker` failed.");
            }
        });
    }

    {
        let tx = tx.clone();
        bus.add_listener::<Dashboard, _, _>(move |event: &MpsMetricsUpdated| {
            if tx.send(IncomingEvent::MpsMetricsUpdated((*event).clone())).is_err() {
                warn!("Sending event to `status_worker` failed.");
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
