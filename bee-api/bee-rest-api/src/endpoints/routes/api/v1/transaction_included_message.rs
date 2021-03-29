// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::endpoints::{
    config::ROUTE_TRANSACTION_INCLUDED_MESSAGE,
    filters::{with_storage, with_tangle},
    path_params::transaction_id,
    permission::has_permission,
    rejection::CustomRejection,
    routes::api::v1::message,
    storage::StorageBackend,
};

use bee_message::{
    output::{CreatedOutput, OutputId},
    payload::transaction::TransactionId,
};
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;
use bee_tangle::MsTangle;

use warp::{reject, Filter, Rejection, Reply};

use std::{net::IpAddr, ops::Deref};

fn path() -> impl Filter<Extract = (TransactionId,), Error = Rejection> + Clone {
    super::path()
        .and(warp::path("transactions"))
        .and(transaction_id())
        .and(warp::path("included-message"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::get())
        .and(has_permission(
            ROUTE_TRANSACTION_INCLUDED_MESSAGE,
            public_routes,
            allowed_ips,
        ))
        .and(with_storage(storage))
        .and(with_tangle(tangle))
        .and_then(transaction_included_message)
}

pub(crate) async fn transaction_included_message<B: StorageBackend>(
    transaction_id: TransactionId,
    storage: ResourceHandle<B>,
    tangle: ResourceHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    // Safe to unwrap since 0 is a valid index;
    let output_id = OutputId::new(transaction_id, 0).unwrap();

    match Fetch::<OutputId, CreatedOutput>::fetch(storage.deref(), &output_id)
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "Can not fetch from storage".to_string(),
            ))
        })? {
        Some(output) => message::message(*output.message_id(), tangle).await,
        None => {
            return Err(reject::custom(CustomRejection::NotFound(
                "Can not find output".to_string(),
            )));
        }
    }
}
