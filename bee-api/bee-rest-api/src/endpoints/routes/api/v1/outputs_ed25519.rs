// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, path_params::ed25519_address, rejection::CustomRejection,
        routes::api::v1::MAX_RESPONSE_RESULTS, storage::StorageBackend, ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::OutputsAddressResponse},
};

use bee_ledger::{
    types::LedgerIndex,
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_message::{address::Ed25519Address, output::OutputId, prelude::Address};

use futures::channel::oneshot;
use log::error;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

fn path() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(ed25519_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|addr, args| async move { outputs_ed25519(addr, args).await })
        .boxed()
}

pub(crate) async fn outputs_ed25519<B: StorageBackend>(
    addr: Ed25519Address,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<Vec<OutputId>>, Error>, LedgerIndex)>();

    if let Err(e) = args
        .consensus_worker
        .send(ConsensusWorkerCommand::FetchOutputs(Address::Ed25519(addr), cmd_tx))
    {
        error!("Request to consensus worker failed: {}.", e);
    }

    let (mut fetched, ledger_index) = match cmd_rx.await.map_err(|e| {
        error!("Response from consensus worker failed: {}.", e);
        reject::custom(CustomRejection::ServiceUnavailable(
            "unable to fetch the outputs of the address".to_string(),
        ))
    })? {
        (Ok(response), ledger_index) => match response {
            Some(ids) => (ids, ledger_index),
            None => (vec![], ledger_index),
        },
        (Err(e), _) => {
            error!("unable to fetch the outputs of the address: {}", e);
            return Err(reject::custom(CustomRejection::ServiceUnavailable(
                "unable to fetch the outputs of the address".to_string(),
            )));
        }
    };

    let count = fetched.len();
    fetched.truncate(MAX_RESPONSE_RESULTS);

    Ok(warp::reply::json(&SuccessBody::new(OutputsAddressResponse {
        address_type: Ed25519Address::KIND,
        address: addr.to_string(),
        max_results: MAX_RESPONSE_RESULTS,
        count,
        output_ids: fetched.iter().map(|id| id.to_string()).collect(),
        ledger_index: *ledger_index,
    })))
}
