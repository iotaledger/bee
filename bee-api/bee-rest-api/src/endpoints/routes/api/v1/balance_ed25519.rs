// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_BALANCE_ED25519, filters::with_storage, path_params::ed25519_address, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::BalanceAddressResponse},
};
use crate::endpoints::filters::with_consensus_worker;
use futures::{channel::oneshot};
use bee_ledger::types::{Balance, LedgerIndex};
use bee_ledger::workers::consensus::{ConsensusWorkerCommand};
use bee_message::address::{Address, Ed25519Address};
use bee_runtime::resource::ResourceHandle;

use log::{warn};
use tokio::sync::mpsc;
use warp::{reject, Filter, Rejection, Reply};

use std::net::IpAddr;
use bee_ledger::workers::error::Error;

fn path() -> impl Filter<Extract = (Ed25519Address,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(ed25519_address())
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
        .and(has_permission(ROUTE_BALANCE_ED25519, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and(with_consensus_worker(consensus_worker))
        .and_then(|addr, storage, consensus_worker| async move { balance_ed25519(addr, storage, consensus_worker).await })
}

pub(crate) async fn balance_ed25519<B: StorageBackend>(
    addr: Ed25519Address,
    storage: ResourceHandle<B>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> Result<impl Reply, Rejection> {

    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<Balance>, Error>, LedgerIndex)>();

    if let Err(e) = consensus_worker.send(ConsensusWorkerCommand::FetchBalance(Address::Ed25519(addr), cmd_tx)) {
        warn!("Sending request to consensus worker failed: {}.", e);
    }

    let response = cmd_rx.await.map_err(|_| {
        warn!("Receiving response from consensus worker failed: {}.", e);
        reject::custom(CustomRejection::ServiceUnavailable(
            "unable to look up the balance of the address".to_string(),
        ))
    })?;

    match response

    match response.map_err(|_| {
        reject::custom(CustomRejection::ServiceUnavailable(
            "can not fetch from storage".to_string(),
        ))
    })?

    match cmd_rx.await.map_err(|_| {
        reject::custom(CustomRejection::ServiceUnavailable(
            "can not fetch from storage".to_string(),
        ))
    }).0? {
        Some(balance) => Ok(warp::reply::json(&SuccessBody::new(BalanceAddressResponse {
            address_type: 1,
            address: addr.to_string(),
            balance: balance.amount(),
            dust_allowed: balance.dust_allowed(),
            ledger_index: response.1,
        }))),
        None => Err(reject::custom(CustomRejection::NotFound(
            "balance not found".to_string(),
        ))),
    }
}
