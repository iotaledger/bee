// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod mock;

use bee_message::{payload::transaction::TransactionId, MessageId};
use bee_network::PeerId;
use bee_vote::{
    error::Error,
    opinion,
    statement::{conflict::Conflict, opinion::Opinion, timestamp::Timestamp},
    Registry,
};

async fn registry(node_id: PeerId, tx_id: TransactionId, msg_id: MessageId) -> Registry {
    let registry = Registry::default();

    registry
        .write_view(node_id, |view| {
            view.add_conflict(Conflict {
                id: tx_id,
                opinion: Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 1,
                },
            });
            view.add_conflict(Conflict {
                id: tx_id,
                opinion: Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 2,
                },
            });
            view.add_timestamp(Timestamp {
                id: msg_id,
                opinion: Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 1,
                },
            });
        })
        .await;

    registry
}

#[tokio::test]
async fn number_entries() {
    let node_id = mock::rand_node_id();
    let tx_id = mock::rand_transaction_id();
    let msg_id = mock::rand_message_id();

    let registry = registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let conflict_opinions = view.get_conflict_opinions(tx_id).unwrap();
            assert_eq!(conflict_opinions.len(), 2);
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn last_entry() {
    let node_id = mock::rand_node_id();
    let tx_id = mock::rand_transaction_id();
    let msg_id = mock::rand_message_id();

    let registry = registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let conflict_opinions = view.get_conflict_opinions(tx_id).unwrap();
            assert_eq!(
                *conflict_opinions.last().unwrap(),
                Opinion {
                    opinion: opinion::Opinion::Like,
                    round: 2
                }
            );
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn not_finalized() {
    let node_id = mock::rand_node_id();
    let tx_id = mock::rand_transaction_id();
    let msg_id = mock::rand_message_id();

    let registry = registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let timestamp_opinions = view.get_timestamp_opinions(msg_id).unwrap();
            assert_eq!(timestamp_opinions.finalized(2), false);
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn finalized() {
    let node_id = mock::rand_node_id();
    let tx_id = mock::rand_transaction_id();
    let msg_id = mock::rand_message_id();

    let registry = registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let conflict_opinions = view.get_conflict_opinions(tx_id).unwrap();
            assert_eq!(conflict_opinions.finalized(2), true);
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn node_not_found() {
    let registry = Registry::default();
    let node_id = mock::rand_node_id();

    assert!(matches!(
        registry.read_view(node_id, |_| {}).await,
        Err(Error::NodeNotFound(_))
    ));
}
