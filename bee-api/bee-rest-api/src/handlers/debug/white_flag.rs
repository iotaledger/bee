// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    filters::CustomRejection::BadRequest,
    handlers::{BodyInner, SuccessBody},
};

use bee_message::{milestone::MilestoneIndex, MessageId};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use warp::{reject, Rejection, Reply};

pub(crate) async fn white_flag(body: JsonValue) -> Result<impl Reply, Rejection> {
    let index_json = &body["index"];
    let parents_json = &body["parentMessageIds"];

    let index = if index_json.is_null() {
        return Err(reject::custom(BadRequest(
            "Invalid index: expected a MilestoneIndex".to_string(),
        )));
    } else {
        MilestoneIndex(
            index_json
                .as_str()
                .ok_or_else(|| reject::custom(BadRequest("Invalid index: expected a MilestoneIndex".to_string())))?
                .parse::<u32>()
                .map_err(|_| reject::custom(BadRequest("Invalid index: expected a MilestoneIndex".to_string())))?,
        )
    };

    let parents: Vec<MessageId> = if parents_json.is_null() {
        return Err(reject::custom(BadRequest(
            "Invalid parents: expected an array of MessageId".to_string(),
        )));
    } else {
        let array = parents_json.as_array().ok_or_else(|| {
            reject::custom(BadRequest(
                "Invalid parents: expected an array of MessageId".to_string(),
            ))
        })?;
        let mut message_ids = Vec::new();
        for s in array {
            let message_id = s
                .as_str()
                .ok_or_else(|| {
                    reject::custom(BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    ))
                })?
                .parse::<MessageId>()
                .map_err(|_| {
                    reject::custom(BadRequest(
                        "Invalid parents: expected an array of MessageId".to_string(),
                    ))
                })?;
            message_ids.push(message_id);
        }
        message_ids
    };

    println!("{:?}", index);
    println!("{:?}", parents);

    Ok(warp::reply::json(&SuccessBody::new(WhiteFlagResponse {
        merkle_tree_hash: String::from("Bee"),
    })))
}

/// Response of GET /debug/whiteflag
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhiteFlagResponse {
    #[serde(rename = "merkleTreeHash")]
    pub merkle_tree_hash: String,
}

impl BodyInner for WhiteFlagResponse {}
