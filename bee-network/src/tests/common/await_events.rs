// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use crate::{Event, GossipReceiver, GossipSender, Multiaddr, NetworkEventReceiver, PeerId};

use tokio::time::{self, Duration};

pub async fn get_bind_address(rx: &mut NetworkEventReceiver) -> Multiaddr {
    let timeout = time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            event = rx.recv() => {
                if let Some(Event::AddressBound { address }) = event {
                    return address;
                }
            },
            () = &mut timeout => {
                panic!("timed out before receiving `AddressBound` event");
            }
        }
    }
}

pub async fn get_local_id(rx: &mut NetworkEventReceiver) -> PeerId {
    let timeout = time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            event = rx.recv() => {
                if let Some(Event::LocalIdCreated { local_id }) = event {
                    return local_id;
                }
            },
            () = &mut timeout => {
                panic!("timed out before receiving `PeerCreated` event");
            }
        }
    }
}

pub async fn get_added_peer_id(rx: &mut NetworkEventReceiver) -> PeerId {
    let timeout = time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            event = rx.recv() => {
                if let Some(Event::PeerAdded { peer_id, .. }) = event {
                    return peer_id;
                }
            },
            () = &mut timeout => {
                panic!("timed out before receiving `PeerAdded` event");
            }
        }
    }
}

pub async fn get_connected_peer_id(rx: &mut NetworkEventReceiver) -> PeerId {
    let timeout = time::sleep(Duration::from_secs(20));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            event = rx.recv() => {
                if let Some(Event::PeerConnected { peer_id, .. }) = event {
                    return peer_id;
                }
            },
            () = &mut timeout => {
                panic!("timed out before receiving `PeerConnected` event");
            }
        }
    }
}

pub async fn get_gossip_channels(rx: &mut NetworkEventReceiver) -> (GossipReceiver, GossipSender) {
    let timeout = time::sleep(Duration::from_secs(20));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            event = rx.recv() => {
                if let Some(Event::PeerConnected { gossip_in, gossip_out, .. }) = event {
                    return (gossip_in, gossip_out);
                }
            },
            () = &mut timeout => {
                panic!("timed out before receiving `PeerConnected` event");
            }
        }
    }
}
