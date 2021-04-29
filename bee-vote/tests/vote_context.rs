// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_test::rand::transaction::rand_transaction_id;
use bee_vote::{
    context::{VoteContextBuilder, VoteObject},
    opinion::{Opinion, Opinions},
};

#[test]
fn is_finalized() {
    let ctx = VoteContextBuilder::new(VoteObject::Conflict(rand_transaction_id()))
        .with_initial_opinions(Opinions::new(vec![Opinion::Like; 5]))
        .build()
        .unwrap();

    assert!(ctx.finalized(2, 2));
}

#[test]
fn is_not_finalized() {
    let ctx = VoteContextBuilder::new(VoteObject::Conflict(rand_transaction_id()))
        .with_initial_opinions(Opinions::new(vec![
            Opinion::Like,
            Opinion::Like,
            Opinion::Like,
            Opinion::Like,
            Opinion::Dislike,
        ]))
        .build()
        .unwrap();

    assert!(!ctx.finalized(2, 2));
}

#[test]
fn last_opinion() {
    let ctx = VoteContextBuilder::new(VoteObject::Conflict(rand_transaction_id()))
        .with_initial_opinions(Opinions::new(vec![Opinion::Like; 4]))
        .build()
        .unwrap();

    assert_eq!(ctx.last_opinion(), Some(Opinion::Like));

    let ctx = VoteContextBuilder::new(VoteObject::Conflict(rand_transaction_id()))
        .with_initial_opinions(Opinions::new(vec![
            Opinion::Like,
            Opinion::Like,
            Opinion::Like,
            Opinion::Dislike,
        ]))
        .build()
        .unwrap();

    assert_eq!(ctx.last_opinion(), Some(Opinion::Dislike));
}
