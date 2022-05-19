// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_block::output::OutputId;
use bee_ledger::{
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;
use futures::channel::oneshot;
use log::error;
use tokio::sync::mpsc;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_OUTPUT_METADATA,
        filters::{with_consensus_worker, with_storage},
        path_params::output_id,
        permission::has_permission,
        rejection::CustomRejection,
        storage::StorageBackend,
    },
    types::responses::OutputMetadataResponse,
};

fn path() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("outputs"))
        .and(output_id())
        .and(warp::path("metadata"))
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
        .and(has_permission(ROUTE_OUTPUT_METADATA, public_routes, allowed_ips))
        .and(with_storage(storage))
        .and(with_consensus_worker(consensus_worker))
        .and_then(|output_id, storage, consensus_worker| async move {
            output_metadata(output_id, storage, consensus_worker).await
        })
        .boxed()
}

pub(crate) fn create_output_metadata(
    output_id: &OutputId,
    created_output: &CreatedOutput,
    consumed_output: Option<&ConsumedOutput>,
    ledger_index: LedgerIndex,
) -> OutputMetadataResponse {
    OutputMetadataResponse {
        block_id: created_output.block_id().to_string(),
        transaction_id: output_id.transaction_id().to_string(),
        output_index: output_id.index(),
        is_spent: consumed_output.is_some(),
        milestone_index_spent: consumed_output.map(|o| *o.milestone_index()),
        milestone_timestamp_spent: consumed_output.map(|o| o.milestone_timestamp()),
        transaction_id_spent: consumed_output.map(|o| o.target().to_string()),
        milestone_index_booked: *created_output.milestone_index(),
        milestone_timestamp_booked: created_output.milestone_timestamp(),
        ledger_index: *ledger_index,
    }
}

pub(crate) async fn output_metadata<B: StorageBackend>(
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
            Some(created_output) => {
                let consumed_output = Fetch::<OutputId, ConsumedOutput>::fetch(&*storage, &output_id).map_err(|e| {
                    error!("unable to fetch the output: {}", e);
                    reject::custom(CustomRejection::ServiceUnavailable(
                        "unable to fetch the output".to_string(),
                    ))
                })?;

                Ok(warp::reply::json(&create_output_metadata(
                    &output_id,
                    &created_output,
                    consumed_output.as_ref(),
                    ledger_index,
                )))
            }
            None => Err(reject::custom(CustomRejection::NotFound(
                "output metadata not found".to_string(),
            ))),
        },
        (Err(e), _) => {
            error!("unable to fetch the output metadata: {}", e);
            Err(reject::custom(CustomRejection::ServiceUnavailable(
                "unable to fetch the output metadata".to_string(),
            )))
        }
    }
}
