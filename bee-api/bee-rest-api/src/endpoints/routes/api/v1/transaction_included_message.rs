// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::CreatedOutput;
use bee_message::{output::OutputId, payload::transaction::TransactionId};
use bee_storage::backend::StorageBackendExt;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::endpoints::{
    filters::with_args, path_params::transaction_id, rejection::CustomRejection, routes::api::v1::message,
    storage::StorageBackend, ApiArgsFullNode,
};

fn path() -> impl Filter<Extract = (TransactionId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("transactions"))
        .and(transaction_id())
        .and(warp::path("included-message"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: ApiArgsFullNode<B>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|transaction_id, args| async move { transaction_included_message(transaction_id, args) })
        .boxed()
}

pub(crate) fn transaction_included_message<B: StorageBackend>(
    transaction_id: TransactionId,
    args: ApiArgsFullNode<B>,
) -> Result<impl Reply, Rejection> {
    // Safe to unwrap since 0 is a valid index;
    let output_id = OutputId::new(transaction_id, 0).unwrap();

    match args.storage.fetch::<OutputId, CreatedOutput>(&output_id).map_err(|_| {
        reject::custom(CustomRejection::ServiceUnavailable(
            "Can not fetch from storage".to_string(),
        ))
    })? {
        Some(output) => message::message(*output.message_id(), args),
        None => Err(reject::custom(CustomRejection::NotFound(
            "Can not find output".to_string(),
        ))),
    }
}
