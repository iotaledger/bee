// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_BALANCE_ED25519, filters::with_consensus_worker, path_params::ed25519_address,
        permission::has_permission, rejection::CustomRejection,
    },
    types::{body::SuccessBody, responses::BalanceAddressResponse},
};

use bee_ledger::{
    types::{Balance, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_message::address::{Address, Ed25519Address};

use futures::channel::oneshot;
use log::error;
use tokio::sync::mpsc;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (Ed25519Address,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(ed25519_address())
        .and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_BALANCE_ED25519, public_routes, allowed_ips))
        .and(with_consensus_worker(consensus_worker))
        .and_then(|addr, consensus_worker| async move { balance_ed25519(addr, consensus_worker).await })
        .boxed()
}

pub(crate) async fn balance_ed25519(
    addr: Ed25519Address,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> Result<impl Reply, Rejection> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<Balance>, Error>, LedgerIndex)>();

    if let Err(e) = consensus_worker.send(ConsensusWorkerCommand::FetchBalance(Address::Ed25519(addr), cmd_tx)) {
        error!("request to consensus worker failed: {}.", e);
    }

    match cmd_rx.await.map_err(|e| {
        error!("response from consensus worker failed: {}.", e);
        reject::custom(CustomRejection::ServiceUnavailable(
            "unable to fetch the balance of the address".to_string(),
        ))
    })? {
        (Ok(response), ledger_index) => match response {
            Some(balance) => Ok(warp::reply::json(&SuccessBody::new(BalanceAddressResponse {
                address_type: Ed25519Address::KIND,
                address: addr.to_string(),
                balance: balance.amount(),
                dust_allowed: balance.dust_allowed(),
                ledger_index: *ledger_index,
            }))),
            None => Err(reject::custom(CustomRejection::NotFound(
                "balance not found".to_string(),
            ))),
        },
        (Err(e), _) => {
            error!("unable to fetch the balance of the address: {}", e);
            Err(reject::custom(CustomRejection::ServiceUnavailable(
                "unable to fetch the balance of the address".to_string(),
            )))
        }
    }
}
