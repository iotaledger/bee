// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    endpoints::{
        config::ROUTE_SUBMIT_MESSAGE_RAW,
        filters::{with_message_submitter, with_tangle},
        permission::has_permission,
        rejection::CustomRejection,
        routes::api::v1::submit_message::forward_to_message_submitter,
        storage::StorageBackend,
    },
    types::{body::SuccessBody, responses::SubmitMessageResponse},
};

use bee_common::packable::Packable;
use bee_message::Message;
use bee_protocol::workers::MessageSubmitterWorkerEvent;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use tokio::sync::mpsc;
use warp::{http::StatusCode, reject, Filter, Rejection, Reply};

use std::net::IpAddr;

fn path() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    super::path().and(warp::path("messages")).and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    self::path()
        .and(warp::post())
        .and(has_permission(ROUTE_SUBMIT_MESSAGE_RAW, public_routes, allowed_ips))
        .and(warp::body::bytes())
        .and(with_tangle(tangle))
        .and(with_message_submitter(message_submitter))
        .and_then(submit_message_raw)
}

pub(crate) async fn submit_message_raw<B: StorageBackend>(
    buf: warp::hyper::body::Bytes,
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> Result<impl Reply, Rejection> {
    let message =
        Message::unpack(&mut &(*buf)).map_err(|e| reject::custom(CustomRejection::BadRequest(format!("can not submit message: invalid bytes provided: the message format is not respected: {}", e))))?;
    let message_id = forward_to_message_submitter(message, tangle, message_submitter).await?;
    Ok(warp::reply::with_status(
        warp::reply::json(&SuccessBody::new(SubmitMessageResponse {
            message_id: message_id.to_string(),
        })),
        StatusCode::CREATED,
    ))
}
