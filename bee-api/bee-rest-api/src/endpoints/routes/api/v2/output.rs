// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use axum::{
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::get,
    Router,
};
use bee_ledger::{
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_message::output::OutputId;
use bee_storage::access::Fetch;
use futures::channel::oneshot;
use log::error;

use crate::{
    endpoints::{error::ApiError, storage::StorageBackend, ApiArgsFullNode},
    types::responses::OutputResponse,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/outputs/:output_id", get(output::<B>))
}

pub(crate) async fn output<B: StorageBackend>(
    Path(output_id): Path<OutputId>,
    Extension(args): Extension<Arc<ApiArgsFullNode<B>>>,
) -> Result<impl IntoResponse, ApiError> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>();

    if let Err(e) = args
        .consensus_worker
        .send(ConsensusWorkerCommand::FetchOutput(output_id, cmd_tx))
    {
        error!("request to consensus worker failed: {}.", e);
    }

    match cmd_rx.await.map_err(|e| {
        error!("response from consensus worker failed: {}.", e);
        ApiError::ServiceUnavailable("unable to fetch the output".to_string())
    })? {
        (Ok(response), ledger_index) => match response {
            Some(output) => {
                let consumed_output = match Fetch::<OutputId, ConsumedOutput>::fetch(&*args.storage, &output_id) {
                    Err(e) => {
                        error!("unable to fetch the output: {}", e);
                        return Err(ApiError::ServiceUnavailable("unable to fetch the output".to_string()));
                    }
                    Ok(output) => output,
                };

                Ok(Json(OutputResponse {
                    message_id: output.message_id().to_string(),
                    transaction_id: output_id.transaction_id().to_string(),
                    output_index: output_id.index(),
                    is_spent: consumed_output.is_some(),
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
            None => Err(ApiError::NotFound("output not found".to_string())),
        },
        (Err(e), _) => {
            error!("unable to fetch the output: {}", e);
            Err(ApiError::ServiceUnavailable("unable to fetch the output".to_string()))
        }
    }
}
