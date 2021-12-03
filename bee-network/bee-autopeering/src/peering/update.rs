// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    filter::NeighborFilter,
    manager::{self, OutboundNeighborhood},
    neighbor::{salt_distance, Neighbor},
};

use crate::{
    delay::ManualDelayFactory,
    discovery::manager::get_verified_peers,
    local::Local,
    peer::lists::ActivePeersList,
    request::RequestManager,
    server::ServerTx,
    task::Repeat,
    time::{self, MINUTE, SECOND},
    NeighborValidator,
};

use std::time::Duration;

/// Outbound neighborhood update interval if there are remaining slots.
#[allow(clippy::identity_op)]
pub(crate) const OPEN_OUTBOUND_NBH_UPDATE_SECS: Duration = Duration::from_secs(1 * SECOND);
/// Outbound neighborhood update interval if there are no remaining slots.
#[allow(clippy::identity_op)]
const FULL_OUTBOUND_NBH_UPDATE_SECS: Duration = Duration::from_secs(1 * MINUTE);

pub(crate) static OUTBOUND_NBH_UPDATE_INTERVAL: ManualDelayFactory =
    ManualDelayFactory::new(OPEN_OUTBOUND_NBH_UPDATE_SECS);

#[derive(Clone)]
pub(crate) struct UpdateContext<V: NeighborValidator> {
    pub(crate) local: Local,
    pub(crate) request_mngr: RequestManager,
    pub(crate) active_peers: ActivePeersList,
    pub(crate) nb_filter: NeighborFilter<V>,
    pub(crate) outbound_nbh: OutboundNeighborhood,
    pub(crate) server_tx: ServerTx,
}

pub(crate) fn do_update<V: NeighborValidator + 'static>() -> Repeat<UpdateContext<V>> {
    Box::new(|ctx| update_outbound(ctx))
}

// Hive.go: updateOutbound updates outbound neighbors.
fn update_outbound<V: NeighborValidator + 'static>(ctx: &UpdateContext<V>) {
    let local_id = ctx.local.peer_id();
    let local_salt = ctx.local.public_salt().expect("missing public salt");

    // TODO: write `get_verified_peers_sorted` which collects verified peers into a BTreeSet
    let verif_peers = get_verified_peers(&ctx.active_peers)
        .into_iter()
        .map(|p| {
            let peer = p.into_peer();
            let peer_id = *peer.peer_id();
            Neighbor::new(peer, salt_distance(&local_id, &peer_id, &local_salt))
        })
        .collect::<Vec<_>>();

    if verif_peers.is_empty() {
        log::trace!("Currently no verified peers.");
        return;
    }

    // Apply the filter to the verified peers to yield a set of neighbor candidates.
    let mut candidates = ctx.nb_filter.apply_list(&verif_peers);

    if candidates.is_empty() {
        log::trace!("Currently no suitable candidates.");
        return;
    }

    // Sort candidats by their distance, so that we start with the closest candidate.
    candidates.sort_unstable();

    // Hive.go: select new candidate
    if let Some(candidate) = ctx.outbound_nbh.select_from_candidate_list(&candidates).cloned() {
        let ctx_ = ctx.clone();

        tokio::spawn(async move {
            if let Some(status) = manager::begin_peering(
                candidate.peer_id(),
                &ctx_.active_peers,
                &ctx_.request_mngr,
                &ctx_.server_tx,
                &ctx_.local,
            )
            .await
            {
                if status {
                    set_outbound_update_interval(&ctx_.outbound_nbh, &ctx_.local);
                } else {
                    ctx_.nb_filter.add(*candidate.peer_id());
                }
            } else {
                ctx_.nb_filter.add(*candidate.peer_id());
            }
        });
    }
}

fn set_outbound_update_interval(outbound_nbh: &OutboundNeighborhood, local: &Local) {
    let mut delay = OPEN_OUTBOUND_NBH_UPDATE_SECS;

    if outbound_nbh.is_full() {
        delay = FULL_OUTBOUND_NBH_UPDATE_SECS
    };

    let salt_expiration = Duration::from_secs(
        time::until(local.public_salt().expect("missing public salt").expiration_time()).expect("time until error"),
    );

    if salt_expiration < delay {
        delay = salt_expiration;
    }

    OUTBOUND_NBH_UPDATE_INTERVAL.set(delay);
}
