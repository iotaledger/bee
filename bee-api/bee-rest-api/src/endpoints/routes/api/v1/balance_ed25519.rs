// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::{
    types::{Balance, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_message::address::{Address, Ed25519Address};
use futures::channel::oneshot;
use log::error;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        filters::with_args, path_params::ed25519_address, rejection::CustomRejection, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::BalanceAddressResponse},
};

fn path() -> impl Filter<Extract = (Ed25519Address,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(ed25519_address())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(balance_ed25519)
        .boxed()
}

pub(crate) async fn balance_ed25519<B: StorageBackend>(
    addr: Ed25519Address,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<Balance>, Error>, LedgerIndex)>();

    if let Err(e) = args
        .consensus_worker
        .send(ConsensusWorkerCommand::FetchBalance(Address::Ed25519(addr), cmd_tx))
    {
        error!("request to consensus worker failed: {}.", e);
    }

    match cmd_rx.await.map_err(|e| {
        error!("response from consensus worker failed: {}.", e);
        reject::custom(CustomRejection::ServiceUnavailable(
            "unable to fetch the balance of the address".to_string(),
        ))
    })? {
        (Ok(response), ledger_index) => {
            let (balance, dust_allowed) = if let Some(balance) = response {
                (balance.amount(), balance.dust_allowed())
            } else {
                (0, false)
            };

            Ok(warp::reply::json(&SuccessBody::new(BalanceAddressResponse {
                address_type: Ed25519Address::KIND,
                address: addr.to_string(),
                balance,
                dust_allowed,
                ledger_index: *ledger_index,
            })))
        }
        (Err(e), _) => {
            error!("unable to fetch the balance of the address: {}", e);
            Err(reject::custom(CustomRejection::ServiceUnavailable(
                "unable to fetch the balance of the address".to_string(),
            )))
        }
    }
}
