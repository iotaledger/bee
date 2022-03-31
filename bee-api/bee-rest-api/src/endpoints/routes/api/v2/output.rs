// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_ledger::{
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_message::output::OutputId;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;
use futures::channel::oneshot;
use log::error;
use tokio::sync::mpsc;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_OUTPUT,
        filters::{with_consensus_worker, with_storage},
        path_params::output_id,
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::responses::OutputResponse,
};

fn path() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("outputs"))
        .and(output_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    storage: ResourceHandle<B>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_OUTPUT, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and(with_consensus_worker(consensus_worker))
        .and_then(
            |output_id, storage, consensus_worker| async move { output(output_id, storage, consensus_worker).await },
        )
        .boxed()
}

pub(crate) async fn output<B: StorageBackend>(
    output_id: OutputId,
    storage: ResourceHandle<B>,
    consensus_worker: mpsc::UnboundedSender<ConsensusWorkerCommand>,
) -> Result<impl Reply, Rejection> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>();

    if let Err(e) = consensus_worker.send(ConsensusWorkerCommand::FetchOutput(output_id, cmd_tx)) {
        error!("request to consensus worker failed: {}.", e);
    }

    match cmd_rx.await.map_err(|e| {
        error!("response from consensus worker failed: {}.", e);
        reject::custom(CustomRejection::ServiceUnavailable(
            "unable to fetch the output".to_string(),
        ))
    })? {
        (Ok(response), ledger_index) => match response {
            Some(output) => {
                let is_spent = Fetch::<OutputId, ConsumedOutput>::fetch(&*storage, &output_id).map_err(|e| {
                    error!("unable to fetch the output: {}", e);
                    reject::custom(CustomRejection::ServiceUnavailable(
                        "unable to fetch the output".to_string(),
                    ))
                })?;

                Ok(warp::reply::json(&OutputResponse {
                    message_id: output.message_id().to_string(),
                    transaction_id: output_id.transaction_id().to_string(),
                    output_index: output_id.index(),
                    is_spent: is_spent.is_some(),
                    // TODO
                    milestone_index_spent: None,
                    // TODO
                    milestone_timestamp_spent: None,
                    // TODO
                    transaction_id_spent: None,
                    // TODO
                    milestone_index_booked: 0,
                    // TODO
                    milestone_timestamp_booked: 0,
                    ledger_index: *ledger_index,
                    output: output.inner().into(),
                }))
            }
            None => Err(reject::custom(CustomRejection::NotFound(
                "output not found".to_string(),
            ))),
        },
        (Err(e), _) => {
            error!("unable to fetch the output: {}", e);
            Err(reject::custom(CustomRejection::ServiceUnavailable(
                "unable to fetch the output".to_string(),
            )))
        }
    }
}
