// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_BALANCE_BECH32, filters::with_consensus_worker, path_params::bech32_address,
    permission::has_permission, routes::api::v1::balance_ed25519::balance_ed25519,
};

use bee_ledger::workers::consensus::ConsensusWorkerCommand;
use bee_message::address::Address;

use tokio::sync::mpsc;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (Address,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(bech32_address())
        .and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_BALANCE_BECH32, public_routes, allowed_ips))
        .and(with_consensus_worker(consensus_worker))
        .and_then(|addr, consensus_worker| async move { balance_bech32(addr, consensus_worker).await })
        .boxed()
}

pub(crate) async fn balance_bech32(
    addr: Address,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => balance_ed25519(a, consensus_worker).await,
    }
}
