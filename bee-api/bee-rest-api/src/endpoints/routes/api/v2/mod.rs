// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod add_peer;
pub mod info;
pub mod messages;
pub mod messages_children;
pub mod messages_metadata;
pub mod messages_raw;
pub mod milestone_by_milestone_id;
pub mod milestone_by_milestone_index;
pub mod outputs;
pub mod peer;
pub mod peers;
pub mod receipts;
pub mod receipts_at;
pub mod remove_peer;
pub mod submit_message;
pub mod tips;
pub mod transaction_included_message;
pub mod treasury;
pub mod utxo_changes_by_milestone_id;
pub mod utxo_changes_by_milestone_index;

use axum::Router;

use crate::endpoints::storage::StorageBackend;

pub(crate) const MAX_RESPONSE_RESULTS: usize = 1000;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().nest(
        "/v2",
        add_peer::filter::<B>()
            .merge(info::filter::<B>())
            .merge(messages::filter::<B>())
            .merge(messages_children::filter::<B>())
            .merge(messages_metadata::filter::<B>())
            .merge(messages_raw::filter::<B>())
            .merge(milestone_by_milestone_id::filter::<B>())
            .merge(milestone_by_milestone_index::filter::<B>())
            .merge(outputs::filter::<B>())
            .merge(peer::filter::<B>())
            .merge(peers::filter::<B>())
            .merge(receipts::filter::<B>())
            .merge(receipts_at::filter::<B>())
            .merge(remove_peer::filter::<B>())
            .merge(submit_message::filter::<B>())
            .merge(tips::filter::<B>())
            .merge(transaction_included_message::filter::<B>())
            .merge(treasury::filter::<B>())
            .merge(utxo_changes_by_milestone_id::filter::<B>())
            .merge(utxo_changes_by_milestone_index::filter::<B>()),
    )
}
