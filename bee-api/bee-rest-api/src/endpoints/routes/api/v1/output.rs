// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        filters::with_args, path_params::output_id, rejection::CustomRejection, storage::StorageBackend,
        ApiArgsFullNode,
    },
    types::{body::SuccessBody, responses::OutputResponse},
};

use bee_ledger::{
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_message::output::OutputId;
use bee_storage::access::Fetch;

use futures::channel::oneshot;
use log::error;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use std::sync::Arc;

fn path() -> impl Filter<Extract = (OutputId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("outputs"))
        .and(output_id())
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgsFullNode<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|output_id, args| async move { output(output_id, args).await })
        .boxed()
}

pub(crate) async fn output<B: StorageBackend>(
    output_id: OutputId,
    args: Arc<ApiArgsFullNode<B>>,
) -> Result<impl Reply, Rejection> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>();

    if let Err(e) = args
        .consensus_worker
        .send(ConsensusWorkerCommand::FetchOutput(output_id, cmd_tx))
    {
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
                let consumed_output = match Fetch::<OutputId, ConsumedOutput>::fetch(&*args.storage, &output_id) {
                    Err(e) => {
                        error!("unable to fetch the output: {}", e);
                        return reject::custom(CustomRejection::ServiceUnavailable(
                            "unable to fetch the output".to_string(),
                        ));
                    }
                    Ok(output) => output,
                };

                let (is_spent, milestone_index_spent, transaction_id_spent) =
                    if let Some(consumed_output) = consumed_output {
                        (
                            true,
                            Some(*consumed_output.index()),
                            Some(consumed_output.target().to_string()),
                        )
                    } else {
                        (false, None, None)
                    };

                Ok(warp::reply::json(&SuccessBody::new(OutputResponse {
                    message_id: output.message_id().to_string(),
                    transaction_id: output_id.transaction_id().to_string(),
                    output_index: output_id.index(),
                    is_spent,
                    output: output.inner().into(),
                    ledger_index: *ledger_index,
                    milestone_index_spent,
                    transaction_id_spent,
                })))
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
