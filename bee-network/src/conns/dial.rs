// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{errors::Error, manager::LISTEN_ADDRESSES, Origin};

use crate::{
    interaction::events::InternalEventSender,
    peers::{BannedAddrList, BannedPeerList, PeerInfo, PeerList, PeerRelation},
    transport::build_transport,
    Multiaddr, PeerId, ShortId,
};

use log::*;

use libp2p::{identity, Transport};

pub async fn dial_peer(
    peer_id: &PeerId,
    local_keys: &identity::Keypair,
    internal_event_sender: &InternalEventSender,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
) -> Result<(), Error> {
    // Prevent dialing oneself.
    if peer_id == &local_keys.public().into_peer_id() {
        return Err(Error::DialedSelf(peer_id.short()));
    }

    // Prevent duplicate connections.
    if let Ok(connected) = peers.is(peer_id, |_, state| state.is_connected()).await {
        if connected {
            return Err(Error::DuplicateConnection(peer_id.short()));
        }
    }

    // Prevent dialing banned peers.
    if banned_peers.contains(peer_id) {
        return Err(Error::DialedBannedPeer(peer_id.short()));
    }

    // Prevent dialing unlisted/unregistered peers.
    let peer_info = peers
        .get_info(peer_id)
        .await
        .map_err(|_| Error::DialedUnlistedPeer(peer_id.short()))?;

    // Prevent dialing banned addresses.
    if banned_addrs.contains(&peer_info.address.to_string()) {
        return Err(Error::DialedBannedAddress(peer_info.address));
    }

    log_dialing_peer(&peer_info);

    let (id, muxer) = build_transport(local_keys)
        .map_err(|_| Error::CreatingTransportFailed)?
        .dial(peer_info.address.clone())
        .map_err(|_| Error::DialingFailed(peer_info.address.clone()))?
        .await
        .map_err(|_| Error::DialingFailed(peer_info.address.clone()))?;

    // Prevent connecting to dishonest peers or peers we have no up-to-date information about.
    if &id != peer_id {
        return Err(Error::PeerIdMismatch {
            expected: peer_id.to_string(),
            received: id.to_string(),
        });
    }

    let peer_id = id;

    log_outbound_connection_success(&peer_info);

    super::upgrade_connection(
        peer_id,
        peer_info,
        muxer,
        Origin::Outbound,
        internal_event_sender.clone(),
    )
    .await?;

    Ok(())
}

#[inline]
fn log_dialing_peer(peer_info: &PeerInfo) {
    info!("Dialing {}...", peer_info.alias);
}

pub async fn dial_address(
    address: &Multiaddr,
    local_keys: &identity::Keypair,
    internal_event_sender: &InternalEventSender,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
) -> Result<(), Error> {
    // Prevent dialing listen addresses.
    if LISTEN_ADDRESSES.read().unwrap().contains(address) {
        return Err(Error::DialedOwnAddress(address.clone()));
    }

    // Prevent dialing banned addresses.
    if banned_addrs.contains(&address.to_string()) {
        return Err(Error::DialedBannedAddress(address.clone()));
    }

    info!("Dialing...");

    let (peer_id, muxer) = build_transport(local_keys)
        .map_err(|_| Error::CreatingTransportFailed)?
        .dial(address.clone())
        .map_err(|_| Error::DialingFailed(address.clone()))?
        .await
        .map_err(|_| Error::DialingFailed(address.clone()))?;

    // Prevent duplicate connections.
    if let Ok(connected) = peers.is(&peer_id, |_, state| state.is_connected()).await {
        if connected {
            return Err(Error::DuplicateConnection(peer_id.short()));
        }
    }

    // Prevent dialing banned peers.
    if banned_peers.contains(&peer_id) {
        return Err(Error::DialedBannedPeer(peer_id.short()));
    }

    let peer_info = if let Ok(peer_info) = peers.get_info(&peer_id).await {
        // If we have this peer id in our peerlist (but are not already connected to it),
        // then we allow the connection.
        peer_info
    } else {
        // We also allow for a certain number of unknown peers.
        let peer_info = PeerInfo {
            address: address.clone(),
            alias: peer_id.short(),
            relation: PeerRelation::Unknown,
        };

        peers
            .accepts(&peer_id, &peer_info)
            .await
            .map_err(|_| Error::DialedRejectedPeer(peer_info.alias.clone()))?;

        info!("Unknown peer {} accepted.", peer_info.alias);

        peer_info
    };

    log_outbound_connection_success(&peer_info);

    super::upgrade_connection(
        peer_id,
        peer_info,
        muxer,
        Origin::Outbound,
        internal_event_sender.clone(),
    )
    .await?;

    Ok(())
}

#[inline]
fn log_outbound_connection_success(peer_info: &PeerInfo) {
    info!("Established (outbound) connection with {}.", peer_info.alias);
}
