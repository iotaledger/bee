// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_vote::{
    statement::{OpinionStatement, OpinionStatements},
    Opinion,
};

#[test]
fn opinions_not_finalized_empty() {
    let opinions = OpinionStatements::new();
    assert!(!opinions.finalized(0));
}

#[test]
fn opinions_not_finalized() {
    let mut opinions = OpinionStatements::new();
    opinions
        .insert(OpinionStatement {
            opinion: Opinion::Dislike,
            round: 0,
        })
        .unwrap();
    opinions
        .insert(OpinionStatement {
            opinion: Opinion::Like,
            round: 1,
        })
        .unwrap();
    opinions
        .insert(OpinionStatement {
            opinion: Opinion::Dislike,
            round: 2,
        })
        .unwrap();

    assert!(!opinions.finalized(1));
}

#[test]
fn opinions_finalized() {
    let mut opinions = OpinionStatements::new();
    opinions
        .insert(OpinionStatement {
            opinion: Opinion::Dislike,
            round: 0,
        })
        .unwrap();
    opinions
        .insert(OpinionStatement {
            opinion: Opinion::Like,
            round: 1,
        })
        .unwrap();
    opinions
        .insert(OpinionStatement {
            opinion: Opinion::Like,
            round: 2,
        })
        .unwrap();

    assert!(opinions.finalized(1));
}
