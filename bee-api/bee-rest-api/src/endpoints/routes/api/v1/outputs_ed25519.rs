// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_OUTPUTS_ED25519, filters::with_consensus_worker, path_params::ed25519_address,
        permission::has_permission, rejection::CustomRejection,
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
use tokio::sync::mpsc;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (Ed25519Address,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("addresses"))
        .and(warp::path("ed25519"))
        .and(ed25519_address())
        .and(warp::path("outputs"))
        .and(warp::path::end())
}

pub(crate) fn filter(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUTS_ED25519, public_routes, allowed_ips))
        .and(with_consensus_worker(consensus_worker))
        .and_then(|addr, consensus_worker| async move { outputs_ed25519(addr, consensus_worker).await })
        .boxed()
}

pub(crate) async fn outputs_ed25519(
    addr: Ed25519Address,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> Result<impl Reply, Rejection> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<Vec<OutputId>>, Error>, LedgerIndex)>();

    if let Err(e) = consensus_worker.send(ConsensusWorkerCommand::FetchOutputs(Address::Ed25519(addr), cmd_tx)) {
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
    let max_results = 1000;
    fetched.truncate(max_results);

    Ok(warp::reply::json(&SuccessBody::new(OutputsAddressResponse {
        address_type: Ed25519Address::KIND,
        address: addr.to_string(),
        max_results,
        count,
        output_ids: fetched.iter().map(|id| id.to_string()).collect(),
        ledger_index: *ledger_index,
    })))
}
