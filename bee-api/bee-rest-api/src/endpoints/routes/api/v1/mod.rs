// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod add_peer;
pub mod balance_bech32;
pub mod balance_ed25519;
pub mod info;
pub mod message;
pub mod message_children;
pub mod message_metadata;
pub mod message_raw;
pub mod messages_find;
pub mod milestone;
pub mod milestone_utxo_changes;
pub mod output;
pub mod outputs_bech32;
pub mod outputs_ed25519;
pub mod peer;
pub mod peers;
pub mod receipts;
pub mod receipts_at;
pub mod remove_peer;
pub mod submit_message;
pub mod tips;
pub mod transaction_included_message;
pub mod treasury;

use crate::endpoints::{storage::StorageBackend, ApiArgs};

use warp::{self, Filter, Rejection, Reply};

use std::sync::Arc;

pub(crate) const MAX_RESPONSE_RESULTS: usize = 1000;

pub(crate) fn path() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    super::path().and(warp::path("v1"))
}

pub(crate) fn filter<B: StorageBackend>(
    args: Arc<ApiArgs<B>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    add_peer::filter(args.clone())
        .or(balance_bech32::filter(args.clone()))
        .or(balance_ed25519::filter(args.clone()))
        .or(info::filter(args.clone()))
        .or(message::filter(args.clone()))
        .or(message_children::filter(args.clone()))
        .or(message_metadata::filter(args.clone()))
        .or(message_raw::filter(args.clone()))
        .or(messages_find::filter(args.clone()))
        .or(milestone::filter(args.clone()))
        .or(milestone_utxo_changes::filter(args.clone()))
        .or(output::filter(args.clone()))
        .or(outputs_bech32::filter(args.clone()))
        .or(outputs_ed25519::filter(args.clone()))
        .or(peer::filter(args.clone()))
        .or(peers::filter(args.clone()))
        .or(receipts::filter(args.clone()))
        .or(receipts_at::filter(args.clone()))
        .or(remove_peer::filter(args.clone()))
        .or(submit_message::filter(args.clone()))
        .or(tips::filter(args.clone()))
        .or(treasury::filter(args.clone()))
        .or(transaction_included_message::filter(args.clone()))
}
