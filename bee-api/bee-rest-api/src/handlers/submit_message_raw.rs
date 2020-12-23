// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::BadRequest,
    handlers::{submit_message::forward_to_message_submitter, SuccessBody},
    storage::Backend,
};

use bee_common::packable::Packable;
use bee_common_pt2::node::ResHandle;
use bee_message::prelude::*;
use bee_protocol::{tangle::MsTangle, MessageSubmitterWorkerEvent};

use tokio::sync::mpsc;
use warp::{http::StatusCode, reject, Buf, Rejection, Reply};

use crate::handlers::submit_message::SubmitMessageResponse;

pub(crate) async fn submit_message_raw<B: Backend>(
    buf: warp::hyper::body::Bytes,
    tangle: ResHandle<MsTangle<B>>,
    message_submitter: mpsc::UnboundedSender<MessageSubmitterWorkerEvent>,
) -> Result<impl Reply, Rejection> {
    let message = Message::unpack(&mut buf.bytes()).map_err(|e| reject::custom(BadRequest(e.to_string())))?;
    let message_id = forward_to_message_submitter(message, tangle, message_submitter).await?;
    Ok(warp::reply::with_status(
        warp::reply::json(&SuccessBody::new(SubmitMessageResponse {
            message_id: message_id.to_string(),
        })),
        StatusCode::CREATED,
    ))
}
