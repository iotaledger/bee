// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use axum::Router;

use crate::endpoints::storage::StorageBackend;

pub mod add_peer;
pub mod info;
pub mod message;
pub mod message_children;
pub mod message_metadata;
pub mod message_raw;
pub mod milestone;
pub mod milestone_utxo_changes;
pub mod output;
pub mod peer;
pub mod peers;
pub mod receipts;
pub mod receipts_at;
pub mod remove_peer;
pub mod submit_message;
pub mod tips;
pub mod transaction_included_message;
pub mod treasury;

pub(crate) fn filter<B: StorageBackend>() -> Router {
    Router::new().nest(
        "/v2",
        add_peer::filter::<B>()
            .merge(info::filter::<B>())
            .merge(message::filter::<B>())
            .merge(message_children::filter::<B>())
            .merge(message_metadata::filter::<B>())
            .merge(message_raw::filter::<B>())
            .merge(milestone::filter::<B>())
            .merge(milestone_utxo_changes::filter::<B>())
            .merge(output::filter::<B>())
            .merge(peer::filter::<B>())
            .merge(peers::filter::<B>())
            .merge(receipts::filter::<B>())
            .merge(receipts_at::filter::<B>())
            .merge(remove_peer::filter::<B>())
            .merge(submit_message::filter::<B>())
            .merge(tips::filter::<B>())
            .merge(transaction_included_message::filter::<B>())
            .merge(treasury::filter::<B>()),
    )
}
