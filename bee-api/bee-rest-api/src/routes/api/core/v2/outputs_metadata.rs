// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_api_types::responses::OutputMetadataResponse;
use bee_block::output::OutputId;
use bee_ledger::{
    types::{ConsumedOutput, CreatedOutput, LedgerIndex},
    workers::{consensus::ConsensusWorkerCommand, error::Error},
};
use bee_storage::access::Fetch;
use futures::channel::oneshot;
use log::error;

use crate::{error::ApiError, extractors::path::CustomPath, storage::StorageBackend, ApiArgsFullNode};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/outputs/:output_id/metadata", get(outputs_metadata::<B>))
}

async fn outputs_metadata<B: StorageBackend>(
    CustomPath(output_id): CustomPath<OutputId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<OutputMetadataResponse, ApiError> {
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

                Ok(create_output_metadata(
                    &output_id,
                    &created_output,
                    consumed_output.as_ref(),
                    ledger_index,
                ))
            }

            None => Err(ApiError::NotFound),
        },
        (Err(e), _) => {
            error!("response from consensus worker failed: {}", e);
            Err(ApiError::InternalServerError)
        }
    }
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
