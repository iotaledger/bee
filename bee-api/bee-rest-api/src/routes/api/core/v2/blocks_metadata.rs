// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::{extract::Extension, routing::get, Router};
use bee_block::{payload::Payload, semantic::ConflictReason, BlockId};

use crate::{
    error::ApiError,
    extractors::path::CustomPath,
    storage::StorageBackend,
    types::{dtos::LedgerInclusionStateDto, responses::BlockMetadataResponse},
    ApiArgsFullNode, CONFIRMED_THRESHOLD,
};

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().route("/blocks/:block_id/metadata", get(block_metadata::<B>))
}

async fn block_metadata<B: StorageBackend>(
    CustomPath(block_id): CustomPath<BlockId>,
    Extension(args): Extension<ApiArgsFullNode<B>>,
) -> Result<BlockMetadataResponse, ApiError> {
    if !args.tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(ApiError::ServiceUnavailable("the node is not synchronized"));
    }

    match args.tangle.get_block_and_metadata(&block_id) {
        Some((block, metadata)) => {
            // TODO: access constants from URTS
            let ybrsi_delta = 8;
            let obrsi_delta = 13;
            let below_max_depth = 15;

            let (
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason,
                should_promote,
                should_reattach,
            ) = {
                let is_solid;
                let referenced_by_milestone_index;
                let milestone_index;
                let ledger_inclusion_state;
                let conflict_reason;
                let should_promote;
                let should_reattach;

                if let Some(milestone) = metadata.milestone_index() {
                    // block is referenced by a milestone
                    is_solid = true;
                    referenced_by_milestone_index = Some(*milestone);

                    if metadata.flags().is_milestone() {
                        milestone_index = Some(*milestone);
                    } else {
                        milestone_index = None;
                    }

                    ledger_inclusion_state = Some(if let Some(Payload::Transaction(_)) = block.payload() {
                        if metadata.conflict() != ConflictReason::None {
                            conflict_reason = Some(metadata.conflict());
                            LedgerInclusionStateDto::Conflicting
                        } else {
                            conflict_reason = None;
                            // maybe not checked by the ledger yet, but still
                            // returning "included". should
                            // `metadata.flags().is_conflicting` return an Option
                            // instead?
                            LedgerInclusionStateDto::Included
                        }
                    } else {
                        conflict_reason = None;
                        LedgerInclusionStateDto::NoTransaction
                    });
                    should_reattach = None;
                    should_promote = None;
                } else if metadata.flags().is_solid() {
                    // block is not referenced by a milestone but solid
                    is_solid = true;
                    referenced_by_milestone_index = None;
                    milestone_index = None;
                    ledger_inclusion_state = None;
                    conflict_reason = None;

                    let cmi = *args.tangle.get_confirmed_milestone_index();
                    // unwrap() of OBRSI/YBRSI is safe since block is solid
                    let (obrsi, ybrsi) = metadata
                        .omrsi_and_ymrsi()
                        .map(|(o, y)| (*o.index(), *y.index()))
                        .unwrap();

                    if (cmi - obrsi) > below_max_depth {
                        should_promote = Some(false);
                        should_reattach = Some(true);
                    } else if (cmi - ybrsi) > ybrsi_delta || (cmi - obrsi) > obrsi_delta {
                        should_promote = Some(true);
                        should_reattach = Some(false);
                    } else {
                        should_promote = Some(false);
                        should_reattach = Some(false);
                    };
                } else {
                    // the block is not referenced by a milestone and not solid
                    is_solid = false;
                    referenced_by_milestone_index = None;
                    milestone_index = None;
                    ledger_inclusion_state = None;
                    conflict_reason = None;
                    should_reattach = Some(true);
                    should_promote = Some(false);
                }

                (
                    is_solid,
                    referenced_by_milestone_index,
                    milestone_index,
                    ledger_inclusion_state,
                    conflict_reason,
                    should_reattach,
                    should_promote,
                )
            };

            Ok(BlockMetadataResponse {
                block_id: block_id.to_string(),
                parents: block.parents().iter().map(BlockId::to_string).collect(),
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason: conflict_reason.map(|c| c as u8),
                white_flag_index: metadata.white_flag_index(),
                should_promote,
                should_reattach,
            })
        }
        None => Err(ApiError::NotFound),
    }
}
