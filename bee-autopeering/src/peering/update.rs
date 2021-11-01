// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    filter::ExclusionFilter,
    manager::{self, InboundNeighborhood, OutboundNeighborhood},
    neighbor::{salted_distance, NeighborDistance},
};

use crate::{
    delay::ManualDelayFactory,
    discovery::get_verified_peers,
    event::EventTx,
    local::{salt, Local},
    peer::{peerlist::ActivePeersList, PeerStore},
    peering::{
        manager::{publish_peering_event, send_drop_peering_request_to_peer},
        protocol::Direction,
    },
    request::RequestManager,
    server::ServerTx,
    task::Repeat,
    time::{self, MINUTE, SECOND},
};

use std::time::Duration;

/// Outbound neighborhood update interval if there are remaining slots.
pub(crate) const OPEN_OUTBOUND_NBH_UPDATE_SECS: Duration = Duration::from_secs(1 * SECOND);
/// Outbound neighborhood update interval if there are no remaining slots.
const FULL_OUTBOUND_NBH_UPDATE_SECS: Duration = Duration::from_secs(1 * MINUTE);

pub(crate) static OUTBOUND_NBH_UPDATE_INTERVAL: ManualDelayFactory =
    ManualDelayFactory::new(OPEN_OUTBOUND_NBH_UPDATE_SECS);

#[derive(Clone)]
pub(crate) struct UpdateContext<S: PeerStore> {
    pub(crate) local: Local,
    pub(crate) peerstore: S,
    pub(crate) request_mngr: RequestManager<S>,
    pub(crate) active_peers: ActivePeersList,
    pub(crate) excl_filter: ExclusionFilter,
    pub(crate) inbound_nbh: InboundNeighborhood,
    pub(crate) outbound_nbh: OutboundNeighborhood,
    pub(crate) server_tx: ServerTx,
    pub(crate) event_tx: EventTx,
}

pub(crate) fn do_update<S: PeerStore + 'static>() -> Repeat<UpdateContext<S>> {
    Box::new(|ctx| update_outbound(ctx))
}

// Hive.go: updateOutbound updates outbound neighbors.
fn update_outbound<S: PeerStore + 'static>(ctx: &UpdateContext<S>) {
    let local_id = ctx.local.read().peer_id().clone();
    let local_salt = ctx.local.read().public_salt().expect("missing public salt").clone();

    // TODO: write `get_verified_peers_sorted` which collects verified peers into a BTreeSet
    let mut disc_peers = get_verified_peers(&ctx.active_peers)
        .into_iter()
        .map(|p| {
            let peer = p.into_peer();
            let peer_id = peer.peer_id().clone();
            NeighborDistance::new(peer, salted_distance(&local_id, &peer_id, &local_salt))
        })
        .collect::<Vec<_>>();

    // Hive.go: sort verified peers by distance
    disc_peers.sort_unstable();

    // TODO: add NeighborValidator as Matcher
    // ctx.excl_filter.write().add_matcher(matcher)

    // Hive.go: filter out current neighbors
    let candidates = ctx.excl_filter.read().apply_list(&disc_peers);

    if candidates.is_empty() {
        log::debug!("Currently no better outbound neighbors available.");
        return;
    }

    // Hive.go: select new candidate
    if let Some(candidate) = ctx
        .outbound_nbh
        .write()
        .select_from_candidate_list(&candidates)
        .cloned()
    {
        let ctx_ = ctx.clone();

        tokio::spawn(async move {
            if let Some(status) = manager::begin_peering_request(
                candidate.peer_id(),
                &ctx_.request_mngr,
                &ctx_.peerstore,
                &ctx_.server_tx,
                &ctx_.local,
            )
            .await
            {
                if status {
                    log::debug!("Peering successfull with {}.", candidate.peer_id());

                    // ```go
                    // if p := m.inbound.RemovePeer(req.Remote.ID()); p != nil {
                    //     m.triggerPeeringEvent(true, req.Remote, false)
                    //     m.dropPeering(p)
                    // } else {
                    //     m.addNeighbor(m.outbound, req)
                    //     m.triggerPeeringEvent(true, req.Remote, true)
                    // }

                    // Hive.go: if the peer is already in inbound, do not add it and remove it from inbound
                    if let Some(peer) = ctx_.inbound_nbh.write().remove_neighbor(candidate.peer_id()) {
                        publish_peering_event(peer.clone(), Direction::Outbound, false, &ctx_.local, &ctx_.event_tx);
                        send_drop_peering_request_to_peer(peer, &ctx_.server_tx, &ctx_.event_tx);
                    } else {
                        if !ctx_.outbound_nbh.write().insert_neighbor(candidate, &ctx_.local) {
                            log::warn!(
                                "Failed to add neighbor to outbound neighborhood after successful peering request"
                            );
                        }
                    }

                    // Hive.go: call updateOutbound again after the given interval
                    set_outbound_update_interval(&ctx_.outbound_nbh, &ctx_.local);
                } else {
                    log::debug!("Peering request denied.");
                    ctx_.excl_filter.write().exclude_peer(candidate.peer_id().clone());
                }
            } else {
                log::debug!("Peering request failed.");
                ctx_.excl_filter.write().exclude_peer(candidate.peer_id().clone());
            }
        });
    }
}

fn set_outbound_update_interval(outbound_nbh: &OutboundNeighborhood, local: &Local) {
    let mut delay = OPEN_OUTBOUND_NBH_UPDATE_SECS;

    if outbound_nbh.read().is_full() {
        delay = FULL_OUTBOUND_NBH_UPDATE_SECS
    };

    let salt_expiration = Duration::from_secs(
        time::until(
            local
                .read()
                .public_salt()
                .expect("missing public salt")
                .expiration_time(),
        )
        .expect("time until error"),
    );

    if salt_expiration < delay {
        delay = salt_expiration;
    }

    OUTBOUND_NBH_UPDATE_INTERVAL.set(delay);
}
