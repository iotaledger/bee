// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::{BadRequest, NotFound},
    handlers::{BodyInner, SuccessBody},
    storage::Backend,
    types::*,
};

use bee_common_pt2::node::ResHandle;
use bee_message::prelude::*;
use bee_protocol::tangle::MsTangle;

use serde::Serialize;
use warp::{reject, Rejection, Reply};

use std::convert::TryFrom;

pub(crate) async fn message<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => Ok(warp::reply::json(&SuccessBody::new(MessageResponse(
            MessageDto::try_from(&*message).map_err(|e| reject::custom(BadRequest(e)))?,
        )))),
        None => Err(reject::custom(NotFound("can not find message".to_string()))),
    }
}

/// Response of GET /api/v1/messages/{message_id}
#[derive(Clone, Debug, Serialize)]
pub struct MessageResponse(pub MessageDto);

impl BodyInner for MessageResponse {}
