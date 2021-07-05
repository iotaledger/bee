// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_BALANCE_BECH32, filters::with_storage, path_params::bech32_address, permission::has_permission,
    routes::api::v1::balance_ed25519::balance_ed25519, storage::StorageBackend,
};

use bee_ledger::workers::consensus::ConsensusWorkerCommand;
use bee_message::address::Address;
use bee_runtime::resource::ResourceHandle;

use tokio::sync::mpsc;
use warp::{Filter, Rejection, Reply};

use std::net::IpAddr;
use crate::endpoints::filters::with_consensus_worker;

fn path() -> impl Filter<Extract = (Address,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(bech32_address())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_BALANCE_BECH32, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and(with_consensus_worker(consensus_worker))
        .and_then(|addr, storage, consensus_worker| async move { balance_bech32(addr, storage, consensus_worker) })
}

pub(crate) fn balance_bech32<B: StorageBackend>(
    addr: Address,
    storage: ResourceHandle<B>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> Result<impl Reply, Rejection> {
    match addr {
        Address::Ed25519(a) => balance_ed25519(a, storage, consensus_worker),
    }
}
