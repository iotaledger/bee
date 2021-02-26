// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    body::SuccessBody,
    handlers::api::v1::submit_message::{forward_to_message_submitter, SubmitMessageResponse},
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_protocol::MessageSubmitterWorkerEvent;
use bee_runtime::resource::ResourceHandle;
use bee_tangle::MsTangle;

use tokio::sync::mpsc;
use warp::{http::StatusCode, reject, Rejection, Reply};

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
