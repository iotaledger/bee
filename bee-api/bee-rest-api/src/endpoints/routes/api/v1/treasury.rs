// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{filters::with_args, rejection::CustomRejection, storage::StorageBackend, ApiArgs},
    types::{body::SuccessBody, responses::TreasuryResponse},
};

use bee_ledger::workers::storage;

use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

use std::sync::Arc;

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("treasury")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(|args| async move { treasury(args) })
        .boxed()
}

pub(crate) fn treasury<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> Result<impl Reply, Rejection> {
    let treasury =
        storage::fetch_unspent_treasury_output(&*args.storage).map_err(|_| CustomRejection::StorageBackend)?;

    Ok(warp::reply::json(&SuccessBody::new(TreasuryResponse {
        milestone_id: treasury.milestone_id().to_string(),
        amount: treasury.inner().amount(),
    })))
}
