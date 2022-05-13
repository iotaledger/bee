// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use bee_block::{payload::Payload, semantic::ConflictReason, BlockId};
use bee_runtime::resource::ResourceHandle;
use bee_tangle::Tangle;
use warp::{filters::BoxedFilter, reject, Filter, Rejection, Reply};

use crate::{
    endpoints::{
        config::ROUTE_MESSAGE_METADATA, filters::with_tangle, path_params::block_id, permission::has_permission,
        rejection::CustomRejection, storage::StorageBackend, CONFIRMED_THRESHOLD,
    },
    types::{dtos::LedgerInclusionStateDto, responses::BlockMetadataResponse},
};

fn path() -> impl Filter<Extract = (BlockId,), Error = warp::Rejection> + Clone {
    super::path()
        .and(warp::path("blocks"))
        .and(block_id())
        .and(warp::path("metadata"))
        .and(warp::path::end())
}

pub(crate) fn filter<B: StorageBackend>(
    public_routes: Box<[String]>,
    allowed_ips: Box<[IpAddr]>,
    tangle: ResourceHandle<Tangle<B>>,
) -> BoxedFilter<(impl Reply,)> {
    self::path()
        .and(warp::get())
        .and(has_permission(ROUTE_MESSAGE_METADATA, public_routes, allowed_ips))
        .and(with_tangle(tangle))
        .and_then(|block_id, tangle| async move { block_metadata(block_id, tangle) })
        .boxed()
}

pub(crate) fn block_metadata<B: StorageBackend>(
    block_id: BlockId,
    tangle: ResourceHandle<Tangle<B>>,
) -> Result<impl Reply, Rejection> {
    if !tangle.is_confirmed_threshold(CONFIRMED_THRESHOLD) {
        return Err(reject::custom(CustomRejection::ServiceUnavailable(
            "the node is not synchronized".to_string(),
        )));
    }

    match tangle.get_block_and_metadata(&block_id) {
        Some((block, metadata)) => {
            // TODO: access constants from URTS
            let ymrsi_delta = 8;
            let omrsi_delta = 13;
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

                    let cmi = *tangle.get_confirmed_milestone_index();

                    // unwrap() of OMRSI/YMRSI is safe since block is solid
                    let (omrsi, ymrsi) = metadata
                        .omrsi_and_ymrsi()
                        .map(|(o, y)| (*o.index(), *y.index()))
                        .unwrap();

                    if (cmi - omrsi) > below_max_depth {
                        should_promote = Some(false);
                        should_reattach = Some(true);
                    } else if (cmi - ymrsi) > ymrsi_delta || (cmi - omrsi) > omrsi_delta {
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

            Ok(warp::reply::json(&BlockMetadataResponse {
                block_id: block_id.to_string(),
                parent_block_ids: block.parents().iter().map(BlockId::to_string).collect(),
                is_solid,
                referenced_by_milestone_index,
                milestone_index,
                ledger_inclusion_state,
                conflict_reason: conflict_reason.map(|c| c as u8),
                should_promote,
                should_reattach,
            }))
        }
        None => Err(reject::custom(CustomRejection::NotFound(
            "can not find block".to_string(),
        ))),
    }
}
