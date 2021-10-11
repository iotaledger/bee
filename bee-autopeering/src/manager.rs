// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::AutopeeringConfig,
    message::{IncomingMessage, OutgoingMessage},
    peer::DiscoveredPeer,
};

use tokio::sync::mpsc;

type Tx = mpsc::UnboundedSender<OutgoingMessage>;
type Rx = mpsc::UnboundedReceiver<IncomingMessage>;

pub(crate) struct AutopeeringManager {
    // For receiving discovery responses
    receiver: Rx,
    // For sending discorvery requests
    sender: Tx,
    // Storage for discovered peers
    store: (),
    // Config
    config: AutopeeringConfig,
}

impl AutopeeringManager {
    pub(crate) fn new(rx: Rx, tx: Tx, config: AutopeeringConfig) -> Self {
        // TODO: read the store
        let store = ();

        Self {
            receiver: rx,
            sender: tx,
            store,
            config,
        }
    }

    pub(crate) async fn run(self) {
        let AutopeeringManager {
            receiver: rx,
            sender: tx,
            store,
            config,
        } = self;

        // contact the entry nodes
    }
}
