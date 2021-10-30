// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    filter::ExclusionFilter,
    manager::{self, InboundNeighborhood, OutboundNeighborhood},
    messages::PeeringRequest,
    neighbor::{NeighborValidator, Neighborhood},
};

use crate::{
    event::EventTx,
    local::Local,
    peer::Peer,
    peering::neighbor::{self, NeighborDistance},
    server::ServerTx,
};

use std::{fmt, iter};

#[derive(Debug)]
pub(crate) enum Direction {
    Inbound,
    Outbound,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self, Direction::Inbound) {
            "in"
        } else {
            "out"
        }
        .fmt(f)
    }
}

pub(crate) fn request_peering(peer_req: PeeringRequest, local: &Local) {
    // let status = handle_in_request(peer_req, peer, local, validator, filter, inbound_nbh, outbound_nbh);

    // Hive.go: trigger in the main loop to guarantee order of events
    // trigger_peering_event(peer, false, local, status)
}

/// TODO: incoming peering request handler?
pub(crate) fn handle_in_request<V: NeighborValidator>(
    peer_req: PeeringRequest,
    peer: Peer,
    local: &Local,
    validator: &V,
    filter: &ExclusionFilter,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
    server_tx: &ServerTx,
    event_tx: &EventTx,
) -> bool {
    if is_valid_neighbor(&peer, local, validator) {
        let distance = neighbor::salted_distance(
            local.read().peer_id(),
            peer.peer_id(),
            local.read().private_salt().expect("missing private salt"),
        );

        let nb_distance = NeighborDistance::new(peer, distance);

        let filter = create_exclusion_filter(local, inbound_nbh, outbound_nbh);

        if filter.read().ok(&nb_distance) {
            if let Some(peer) = inbound_nbh.write().select(nb_distance) {
                add_replace_neighbor(peer, local, &inbound_nbh, server_tx, event_tx);
                return true;
            }
        }
    }
    false
}

/// Determins when a peer is a valid neighbor.
pub(crate) fn is_valid_neighbor<V: NeighborValidator>(peer: &Peer, local: &Local, validator: &V) -> bool {
    local.read().peer_id() != peer.peer_id() && validator.is_valid(peer)
}

/// Adds a neighbor to a neighborhood. Possibly even replaces the so far furthest neighbor.
pub(crate) fn add_replace_neighbor<const S: usize, const B: bool>(
    peer: Peer,
    local: &Local,
    nbh: &Neighborhood<S, B>,
    server_tx: &ServerTx,
    event_tx: &EventTx,
) {
    // Hive.go: drop furthest neighbor if necessary
    if let Some(peer) = nbh.write().remove_furthest() {
        manager::send_drop_peering_request_to_peer(peer, server_tx, event_tx);
    }

    nbh.write().insert_neighbor(peer, local);
}

/// Creates the current exclusion filter.
pub(crate) fn create_exclusion_filter(
    local: &Local,
    inbound_nbh: &InboundNeighborhood,
    outbound_nbh: &OutboundNeighborhood,
) -> ExclusionFilter {
    let mut filter = ExclusionFilter::new();

    // The exclusion filter consists of the local peer and the members of both neighborhoods.
    filter.write().exclude_peers(
        iter::once(local.read().peer_id().clone())
            .chain(inbound_nbh.read().iter().map(|p| p.peer_id()).cloned())
            .chain(outbound_nbh.read().iter().map(|p| p.peer_id()).cloned()),
    );

    filter
}
