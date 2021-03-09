// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::SuccessBody,
    config::ROUTE_SUBMIT_MESSAGE_RAW,
    endpoints::api::v1::submit_message::{forward_to_message_submitter, SubmitMessageResponse},
    filters::{with_message_submitter, with_tangle},
    permission::has_permission,
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_common::packable::Packable;
use bee_message::Message;
use bee_protocol::MessageSubmitterWorkerEvent;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use tokio::sync::mpsc;
use warp::{Filter, http::StatusCode, reject, Rejection, Reply};

use std::net::IpAddr;

pub(crate) fn submit_message_raw_filter<B: StorageBackend>(
    public_routes: Vec<String>,
    allowed_ips: Vec<IpAddr>,
    tangle: ResourceHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("messages"))
        .and(warp::path::end())
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
    let bytes = (*buf).to_vec();
    let message = Message::unpack(&mut bytes.as_slice())
        .map_err(|e| reject::custom(CustomRejection::BadRequest(e.to_string())))?;
    let message_id = forward_to_message_submitter(message, tangle, message_submitter).await?;
    Ok(warp::reply::with_status(
        warp::reply::json(&SuccessBody::new(SubmitMessageResponse {
            message_id: message_id.to_string(),
        })),
        StatusCode::CREATED,
    ))
}
