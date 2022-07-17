// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_api_types::responses::OutputResponse;
use bee_block::output::OutputId;
use bee_ledger::{
    consensus::ConsensusWorkerCommand,
    error::Error,
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
};
use bee_storage::access::Fetch;
use futures::channel::oneshot;
use log::error;

use super::outputs_metadata::create_output_metadata;
use crate::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/outputs/:output_id", get(outputs::<B>))
}

async fn outputs<B: StorageBackend>(
    CustomPath(output_id): CustomPath<OutputId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<OutputResponse, ApiError> {
    let (cmd_tx, cmd_rx) = oneshot::channel::<(Result<Option<CreatedOutput>, Error>, LedgerIndex)>();

    if let Err(e) = args
        .consensus_worker
        .send(ConsensusWorkerCommand::FetchOutput(output_id, cmd_tx))
    {
        error!("request to consensus worker failed: {}", e);
        return Err(ApiError::InternalServerError);
    }

    let consensus_worker_response = cmd_rx.await.map_err(|e| {
        error!("response from consensus worker failed: {}", e);
        ApiError::InternalServerError
    })?;

    match consensus_worker_response {
        (Ok(response), ledger_index) => match response {
            Some(created_output) => {
                let consumed_output =
                    Fetch::<OutputId, ConsumedOutput>::fetch(&*args.storage, &output_id).map_err(|e| {
                        error!("cannot fetch from storage: {}", e);
                        ApiError::InternalServerError
                    })?;

                Ok(OutputResponse {
                    metadata: create_output_metadata(
                        &output_id,
                        &created_output,
                        consumed_output.as_ref(),
                        ledger_index,
                    ),
                    output: created_output.inner().into(),
                })
            }

            None => Err(ApiError::NotFound),
        },
        (Err(e), _) => {
            error!("response from consensus worker failed: {}", e);
            Err(ApiError::InternalServerError)
        }
    }
}
