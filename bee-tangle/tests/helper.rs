// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId};
use bee_tangle::Tangle;
use bee_test::transaction::{create_random_attached_tx, create_random_tx};

pub struct Messages {
    pub a: Message,
    pub b: Message,
    pub c: Message,
    pub d: Message,
    pub e: Message,
}

pub struct MessageIds {
    pub a_hash: MessageId,
    pub b_hash: MessageId,
    pub c_hash: MessageId,
    pub d_hash: MessageId,
    pub e_hash: MessageId,
}

#[allow(clippy::many_single_char_names)]
pub fn create_test_tangle() -> (Tangle<()>, Messages, MessageIds) {
    // a   b
    // |\ /
    // | c
    // |/|
    // d |
    //  \|
    //   e

    pollster::block_on(async {
        let tangle = Tangle::default();

        let (a_hash, a) = create_random_tx();
        let (b_hash, b) = create_random_tx();
        let (c_hash, c) = create_random_attached_tx(a_hash, b_hash);
        let (d_hash, d) = create_random_attached_tx(a_hash, c_hash);
        let (e_hash, e) = create_random_attached_tx(d_hash, c_hash);

        assert_eq!(*c.parent1(), b_hash);
        assert_eq!(*c.parent2(), a_hash);
        assert_eq!(*d.parent1(), c_hash);
        assert_eq!(*d.parent2(), a_hash);
        assert_eq!(*e.parent1(), c_hash);
        assert_eq!(*e.parent2(), d_hash);

        tangle.insert(a_hash, a.clone(), ()).await;
        tangle.insert(b_hash, b.clone(), ()).await;
        tangle.insert(c_hash, c.clone(), ()).await;
        tangle.insert(d_hash, d.clone(), ()).await;
        tangle.insert(e_hash, e.clone(), ()).await;

        assert_eq!(*tangle.get(&c_hash).await.unwrap().parent1(), b_hash);
        assert_eq!(*tangle.get(&c_hash).await.unwrap().parent2(), a_hash);
        assert_eq!(*tangle.get(&d_hash).await.unwrap().parent1(), c_hash);
        assert_eq!(*tangle.get(&d_hash).await.unwrap().parent2(), a_hash);
        assert_eq!(*tangle.get(&e_hash).await.unwrap().parent1(), c_hash);
        assert_eq!(*tangle.get(&e_hash).await.unwrap().parent2(), d_hash);

        // TODO ensure children reference their parents correctly

        assert_eq!(5, tangle.len());
        assert_eq!(2, tangle.num_children(&a_hash));
        assert_eq!(1, tangle.num_children(&b_hash));
        assert_eq!(2, tangle.num_children(&c_hash));
        assert_eq!(1, tangle.num_children(&d_hash));
        assert_eq!(0, tangle.num_children(&e_hash));

        (
            tangle,
            Messages { a, b, c, d, e },
            MessageIds {
                a_hash,
                b_hash,
                c_hash,
                d_hash,
                e_hash,
            },
        )
    })
}
