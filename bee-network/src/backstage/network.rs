// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::event::NetworkEvent;

use crate::{
    backstage::peer::{PeerReaderWorker, PeerWriterWorker},
    config::{NetworkConfig, ManualPeeringConfig},
    network::Network,
};

use backstage::core::{AbortableUnboundedChannel, Actor, ActorError, ActorResult, Rt, StreamExt, SupHandle};

use std::sync::Arc;

/// A network worker.
#[derive(Default)]
pub struct NetworkWorker {}

impl NetworkWorker {
    /// Create a new network worker.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl<S: SupHandle<Self>> Actor<S> for NetworkWorker {
    type Data = (NetworkConfig, ManualPeeringConfig);

    type Channel = AbortableUnboundedChannel<NetworkEvent>;

    async fn init(&mut self, rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        let parent_id = rt
            .parent_id()
            .ok_or_else(|| ActorError::aborted_msg("network worker has no parent"))?;

        rt.lookup(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("network configuration is not available"))
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, config: Self::Data) -> ActorResult<()> {
        let (network_config, manual_peering_config) = config;
        let handle = rt.handle().clone();

        Network::start(network_config, manual_peering_config, move |event| {
            if let Err(err) = handle.send(event) {
                log::warn!("could not publish event: {}", err)
            }
        })
        .await
        .map_err(ActorError::aborted)?;

        while let Some(event) = rt.inbox_mut().next().await {
            match event {
                NetworkEvent::PeerConnected(peer) => {
                    log::debug!("peer {} connected", peer.id());

                    let info = Arc::new(peer.info);
                    let id = info.id();

                    let reader = PeerReaderWorker::new(peer.reader, info.clone());
                    let writer = PeerWriterWorker::new(peer.writer, info);

                    rt.start(Some(format!("{}_reader", id)), reader).await?;
                    rt.start(Some(format!("{}_writer", id)), writer).await?;
                }
                NetworkEvent::PeerWorkerEol | NetworkEvent::PeerWorkerReport => {
                    // TODO: handle status report for peers
                }
            }
        }

        Ok(())
    }
}
