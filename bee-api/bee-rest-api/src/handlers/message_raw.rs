// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{filters::CustomRejection::NotFound, storage::Backend};
use bee_common::{node::ResHandle, packable::Packable};
use bee_message::prelude::*;
use bee_protocol::tangle::MsTangle;
use warp::{http::Response, reject, Rejection, Reply};

pub async fn message_raw<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(message.pack_new())),
        None => Err(reject::custom(NotFound("can not find message".to_string()))),
    }
}
