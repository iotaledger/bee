// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_vote::{
    statement::{OpinionStatement, OpinionStatements},
    Error,
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

#[test]
fn opinion_packed_len() {
    let opinion = OpinionStatement {
        opinion: Opinion::Like,
        round: 0,
    };

    let packed = opinion.pack_new();

    assert_eq!(packed.len(), opinion.packed_len());
    assert_eq!(packed.len(), 2);
}

#[test]
fn opinions_empty() {
    let mut opinions = OpinionStatements::new();
    let opinion = OpinionStatement {
        opinion: Opinion::Like,
        round: 0,
    };

    assert!(opinions.is_empty());

    opinions.insert(opinion).unwrap();

    assert!(!opinions.is_empty());

    opinions.clear();

    assert!(opinions.is_empty());
}

#[test]
fn duplicate_opinion() {
    let mut opinions = OpinionStatements::new();
    let opinion = OpinionStatement {
        opinion: Opinion::Like,
        round: 0,
    };

    opinions.insert(opinion).unwrap();
    
    assert!(matches!(opinions.insert(opinion), Err(Error::DuplicateOpinionStatement(_))));
}
