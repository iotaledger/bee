// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod helper;

use self::helper::*;

use bee_tangle::traversal::*;

#[test]
fn visit_children_follow_parent1_in_simple_graph() {
    // a   b0
    // |\ /
    // | c1
    // |/|
    // d2|
    //  \|
    //   e2

    let (tangle, Messages { b, c, d, e, .. }, MessageIds { b_message_id, .. }) = create_test_tangle();

    let mut txs = vec![];

    visit_children_follow_parent1(&tangle, b_message_id, |_, _| true, |_, tx, _| txs.push(tx.clone()));

    assert_eq!(4, txs.len());

    assert_eq!(b.address(), txs[0].address());
    assert_eq!(c.address(), txs[1].address());
    assert!(d.address() == txs[2].address() || d.address() == txs[3].address());
    assert!(e.address() == txs[2].address() || e.address() == txs[3].address());
}

#[test]
fn visit_parents_follow_parent1_in_simple_graph() {
    // a   b2
    // |\ /
    // | c1
    // |/|
    // d |
    //  \|
    //   e0

    let (
        tangle,
        Messages { e, b, c, d, .. },
        MessageIds {
            d_message_id,
            e_message_id,
            ..
        },
    ) = create_test_tangle();

    let mut txs = vec![];

    visit_parents_follow_parent1(&tangle, e_message_id, |_, _| true, |_, tx, _| txs.push(tx.clone()));

    assert_eq!(3, txs.len());

    assert_eq!(e.address(), txs[0].address());
    assert_eq!(c.address(), txs[1].address());
    assert_eq!(b.address(), txs[2].address());

    txs.clear();

    // a   b2
    // |\ /
    // | c1
    // |/|
    // d0|
    //  \|
    //   e
    visit_parents_follow_parent1(&tangle, d_message_id, |_, _| true, |_, tx, _| txs.push(tx.clone()));

    assert_eq!(d.address(), txs[0].address());
    assert_eq!(c.address(), txs[1].address());
    assert_eq!(b.address(), txs[2].address());
}

#[test]
fn visit_parents_depth_first_in_simple_graph() {
    // a2  b4
    // |\ /
    // | c3
    // |/|
    // d1|
    //  \|
    //   e0

    let (tangle, Messages { a, b, c, d, e, .. }, MessageIds { e_message_id, .. }) = create_test_tangle();

    let mut addresses = vec![];

    visit_parents_depth_first(
        &tangle,
        e_message_id,
        |_, _, _| true,
        |_, data, _| addresses.push(data.address().clone()),
        |_, _, _| {},
        |_| (),
    );

    assert_eq!(5, addresses.len());

    assert_eq!(*e.address(), addresses[0]);
    assert_eq!(*d.address(), addresses[1]);
    assert_eq!(*a.address(), addresses[2]);
    assert_eq!(*c.address(), addresses[3]);
    assert_eq!(*b.address(), addresses[4]);
}
