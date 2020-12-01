// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{errors::Error, Origin};

use crate::{
    interaction::events::InternalEventSender,
    peers::{BannedAddrList, BannedPeerList, PeerInfo, PeerList, PeerRelation, PeerState},
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
    // Prevent duplicate connections.
    if let Ok(connected) = peers.is(peer_id, |_, state| state.is_connected()) {
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
        .map_err(|_| Error::DialedUnlistedPeer(peer_id.short()))?;

    // Prevent dialing banned addresses.
    if banned_addrs.contains(&peer_info.address.to_string()) {
        return Err(Error::DialedBannedAddress(peer_info.address));
    }

    log_dialing_peer(peer_id, &peer_info);

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

    log_outbound_connection_success(&peer_id, &peer_info);

    super::spawn_connection_handler(
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
fn log_dialing_peer(peer_id: &PeerId, peer_info: &PeerInfo) {
    if let Some(alias) = peer_info.alias.as_ref() {
        info!("Dialing '{}/{}'...", alias, peer_id.short());
    } else {
        info!("Dialing '{}'...", peer_id.short());
    }
}

pub async fn dial_address(
    address: &Multiaddr,
    local_keys: &identity::Keypair,
    internal_event_sender: &InternalEventSender,
    peers: &PeerList,
    banned_addrs: &BannedAddrList,
    banned_peers: &BannedPeerList,
) -> Result<(), Error> {
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
    if let Ok(connected) = peers.is(&peer_id, |_, state| state.is_connected()) {
        if connected {
            return Err(Error::DuplicateConnection(peer_id.short()));
        }
    }

    // Prevent dialing banned peers.
    if banned_peers.contains(&peer_id) {
        return Err(Error::DialedBannedPeer(peer_id.short()));
    }

    let peer_info = if let Ok(peer_info) = peers.get_info(&peer_id) {
        // If we have this peer id in our peerlist (but are not already connected to it),
        // then we allow the connection.
        peer_info
    } else {
        // We also allow for a certain number of unknown peers.
        let peer_info = PeerInfo {
            address: address.clone(),
            alias: None,
            relation: PeerRelation::Unknown,
        };

        peers
            .insert(peer_id.clone(), peer_info.clone(), PeerState::Disconnected)
            .map_err(|_| Error::DialedRejectedPeer(peer_id.short()))?;

        info!("Allowing connection to unknown peer '{}'", peer_id.short(),);

        peer_info
    };

    log_outbound_connection_success(&peer_id, &peer_info);

    super::spawn_connection_handler(
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
fn log_outbound_connection_success(peer_id: &PeerId, peer_info: &PeerInfo) {
    if let Some(alias) = peer_info.alias.as_ref() {
        info!(
            "Established (outbound) connection with '{}/{}'.",
            alias,
            peer_id.short(),
        )
    } else {
        info!("Established (outbound) connection with '{}'.", peer_id.short(),);
    }
}
