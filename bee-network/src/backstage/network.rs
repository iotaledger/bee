// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub use crate::event::NetworkEvent;

use crate::{
    backstage::peer::{PeerReaderActor, PeerWriterActor},
    network::Network,
};

use backstage::core::{AbortableUnboundedChannel, Actor, ActorError, ActorResult, Rt, StreamExt, SupHandle};

use std::sync::Arc;

/// A network actor.
#[derive(Default)]
pub struct NetworkActor {}

impl NetworkActor {
    /// Create a new network actor.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl<S: SupHandle<Self>> Actor<S> for NetworkActor {
    type Data = ();

    type Channel = AbortableUnboundedChannel<NetworkEvent>;

    async fn init(&mut self, rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        let parent_id = rt
            .parent_id()
            .ok_or_else(|| ActorError::aborted_msg("network actor has no parent"))?;

        let (network_config, manual_peering_config) = rt
            .lookup(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("network configuration is not available"))?;

        let handle = rt.handle().clone();

        Network::start(network_config, manual_peering_config, move |event| {
            if let Err(err) = handle.send(event) {
                log::warn!("could not publish event: {}", err)
            }
        })
        .await
        .map_err(ActorError::aborted)?;

        Ok(())
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, _: Self::Data) -> ActorResult<()> {
        while let Some(event) = rt.inbox_mut().next().await {
            match event {
                NetworkEvent::PeerConnected(peer) => {
                    log::debug!("peer {} connected", peer.id());

                    let info = Arc::new(peer.info);
                    let id = info.id();

                    let reader = PeerReaderActor::new(peer.reader, info.clone());
                    let writer = PeerWriterActor::new(peer.writer, info);

                    rt.start(Some(format!("{}_reader", id)), reader).await?;
                    rt.start(Some(format!("{}_writer", id)), writer).await?;
                }
                NetworkEvent::PeerActorEol | NetworkEvent::PeerActorReport => {
                    // TODO: handle status report for peers
                }
            }
        }

        Ok(())
    }
}
