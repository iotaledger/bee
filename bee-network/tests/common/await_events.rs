// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use bee_network::{Event, Multiaddr, NetworkEventReceiver, PeerId};

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
                assert!(false, "timed out before receiving `AddressBound` event");
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
                if let Some(Event::LocalIdCreated { peer_id }) = event {
                    return peer_id;
                }
            },
            () = &mut timeout => {
                assert!(false, "timed out before receiving `PeerCreated` event");
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
                assert!(false, "timed out before receiving `PeerAdded` event");
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
                assert!(false, "timed out before receiving `PeerConnected` event");
            }
        }
    }
}
