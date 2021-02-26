// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    handlers::{BodyInner, SuccessBody},
    rejection::CustomRejection,
    storage::StorageBackend,
};

use bee_ledger::model::OutputDiff;
use bee_message::prelude::*;
use bee_runtime::resource::ResourceHandle;
use bee_storage::access::Fetch;

use serde::{Deserialize, Serialize};
use warp::{reject, Rejection, Reply};

use std::ops::Deref;

pub(crate) async fn milestone_utxo_changes<B: StorageBackend>(
    index: MilestoneIndex,
    storage: ResourceHandle<B>,
) -> Result<impl Reply, Rejection> {
    let fetched = match Fetch::<MilestoneIndex, OutputDiff>::fetch(storage.deref(), &index)
        .await
        .map_err(|_| {
            reject::custom(CustomRejection::ServiceUnavailable(
                "can not fetch from storage".to_string(),
            ))
        })? {
        Some(diff) => diff,
        None => {
            return Err(reject::custom(CustomRejection::NotFound(
                "can not find UTXO changes for given milestone index".to_string(),
            )))
        }
    };
    Ok(warp::reply::json(&SuccessBody::new(MilestoneUtxoChanges {
        index: *index,
        created_outputs: fetched.created_outputs().iter().map(|id| id.to_string()).collect(),
        consumed_outputs: fetched.consumed_outputs().iter().map(|id| id.to_string()).collect(),
    })))
}

/// Response of GET /api/v1/milestone/{milestone_index}/utxo-changes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneUtxoChanges {
    pub index: u32,
    #[serde(rename = "createdOutputs")]
    pub created_outputs: Vec<String>,
    #[serde(rename = "consumedOutputs")]
    pub consumed_outputs: Vec<String>,
}

impl BodyInner for MilestoneUtxoChanges {}
