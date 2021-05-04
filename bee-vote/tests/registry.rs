// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::transaction::TransactionId, MessageId};
use bee_network::PeerId;
use bee_test::rand::{message::rand_message_id, transaction::rand_transaction_id};
use bee_vote::{
    error::Error,
    opinion::QueryObjects,
    statement::{Conflict, OpinionStatement, Timestamp},
    Opinion, Opinions, Registry, VoteObject,
};

use std::time::Duration;

async fn create_registry(node_id: PeerId, tx_id: TransactionId, msg_id: MessageId) -> Registry {
    let registry = Registry::default();

    registry
        .write_view(node_id, |view| {
            view.add_conflict(Conflict {
                id: tx_id,
                opinion: OpinionStatement {
                    opinion: Opinion::Like,
                    round: 1,
                },
            })
            .unwrap();
            view.add_conflict(Conflict {
                id: tx_id,
                opinion: OpinionStatement {
                    opinion: Opinion::Like,
                    round: 2,
                },
            })
            .unwrap();
            view.add_timestamp(Timestamp {
                id: msg_id,
                opinion: OpinionStatement {
                    opinion: Opinion::Like,
                    round: 1,
                },
            })
            .unwrap();
        })
        .await;

    registry
}

#[tokio::test]
async fn number_entries() {
    let node_id = PeerId::random();
    let tx_id = rand_transaction_id();
    let msg_id = rand_message_id();

    let registry = create_registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let conflict_opinions = view.get_conflict_opinions(tx_id);
            assert_eq!(conflict_opinions.len(), 2);
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn last_entry() {
    let node_id = PeerId::random();
    let tx_id = rand_transaction_id();
    let msg_id = rand_message_id();

    let registry = create_registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let conflict_opinions = view.get_conflict_opinions(tx_id);
            assert_eq!(
                *conflict_opinions.last().unwrap(),
                OpinionStatement {
                    opinion: Opinion::Like,
                    round: 2
                }
            );
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn not_finalized() {
    let node_id = PeerId::random();
    let tx_id = rand_transaction_id();
    let msg_id = rand_message_id();

    let registry = create_registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let timestamp_opinions = view.get_timestamp_opinions(msg_id);
            assert_eq!(timestamp_opinions.finalized(2), false);
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn finalized() {
    let node_id = PeerId::random();
    let tx_id = rand_transaction_id();
    let msg_id = rand_message_id();

    let registry = create_registry(node_id, tx_id, msg_id).await;

    registry
        .read_view(node_id, |view| {
            let conflict_opinions = view.get_conflict_opinions(tx_id);
            assert_eq!(conflict_opinions.finalized(2), true);
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn node_not_found() {
    let registry = Registry::default();
    let node_id = PeerId::random();

    assert!(matches!(
        registry.read_view(node_id, |_| {}).await,
        Err(Error::NodeNotFound(_)),
    ));
}

#[tokio::test]
async fn clean() {
    let node_id = PeerId::random();
    let tx_id = rand_transaction_id();
    let msg_id = rand_message_id();

    let registry = create_registry(node_id, tx_id, msg_id).await;

    registry.clean(Duration::from_millis(500)).await;

    assert!(registry.read_view(node_id, |_| {}).await.is_ok());
    
    tokio::time::sleep(Duration::from_millis(1000)).await;
    registry.clean(Duration::from_millis(500)).await;

    assert!(matches!(
        registry.read_view(node_id, |_| {}).await, 
        Err(Error::NodeNotFound(_)),
    ));
}

#[tokio::test]
async fn query_entry() {
    let node_id = PeerId::random();
    let tx_id = rand_transaction_id();
    let msg_id = rand_message_id();

    let registry = create_registry(node_id, tx_id, msg_id).await;

    registry.read_view(node_id, |view| {
        let queried_opinions = view.query(QueryObjects {
            conflict_objects: vec![VoteObject::Conflict(tx_id)],
            timestamp_objects: vec![VoteObject::Timestamp(msg_id)],            
        })
        .unwrap();

        assert_eq!(queried_opinions, Opinions::new(vec![Opinion::Like, Opinion::Like]));
    })
    .await
    .unwrap();
}
