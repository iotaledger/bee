// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{filters::with_args, routes::health, storage::StorageBackend, ApiArgs},
    types::{body::SuccessBody, responses::InfoResponse},
};

use warp::{filters::BoxedFilter, Filter, Reply};

use std::{convert::Infallible, sync::Arc};

fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("info")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(with_args(args))
        .and_then(info)
        .boxed()
}

pub(crate) async fn info<B: StorageBackend>(args: Arc<ApiArgs<B>>) -> Result<impl Reply, Infallible> {
    let latest_milestone_index = args.tangle.get_latest_milestone_index();
    let latest_milestone_timestamp = args
        .tangle
        .get_milestone(latest_milestone_index)
        .await
        .map(|m| m.timestamp())
        .unwrap_or_default();

    Ok(warp::reply::json(&SuccessBody::new(InfoResponse {
        name: args.node_info.name.clone(),
        version: args.node_info.version.clone(),
        is_healthy: health::is_healthy(&args.tangle, &args.peer_manager).await,
        network_id: args.network_id.0.to_owned(),
        bech32_hrp: args.bech32_hrp.to_owned(),
        min_pow_score: args.protocol_config.minimum_pow_score(),
        messages_per_second: 0f64,            // TODO
        referenced_messages_per_second: 0f64, // TODO
        referenced_rate: 0f64,                // TODO
        latest_milestone_timestamp,
        latest_milestone_index: *latest_milestone_index,
        confirmed_milestone_index: *args.tangle.get_confirmed_milestone_index(),
        pruning_index: *args.tangle.get_pruning_index(),
        features: {
            let mut features = Vec::new();
            if args.rest_api_config.feature_proof_of_work() {
                features.push("PoW".to_string())
            }
            features
        },
    })))
}
