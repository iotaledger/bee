// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};
use bee_tangle::{Tangle, TangleWalker, TangleWalkerStatus};
use bee_test::rand::message::{metadata::rand_message_metadata, rand_message_with_parents_ids};

use std::collections::HashMap;

fn new_sep(tangle: &Tangle) -> (Message, MessageMetadata, MessageId) {
    new_node(tangle, vec![MessageId::null()])
}

fn new_node(tangle: &Tangle, parents_ids: Vec<MessageId>) -> (Message, MessageMetadata, MessageId) {
    let message = rand_message_with_parents_ids(parents_ids);
    let metadata = rand_message_metadata();
    let message_id = message.id();

    tangle.insert(message_id, message.clone(), metadata.clone());

    (message, metadata, message_id)
}

struct TangleBuilder {
    nodes: Vec<usize>,
    parents: HashMap<usize, Vec<usize>>,
}

impl TangleBuilder {
    fn new() -> Self {
        Self {
            nodes: vec![],
            parents: HashMap::new(),
        }
    }

    fn add_node<const M: usize>(&mut self, node: usize, parents: [usize; M]) -> &mut Self {
        if let Err(i) = self.nodes.binary_search(&node) {
            self.nodes.insert(i, node);
        }

        for parent in parents {
            if let Err(i) = self.nodes.binary_search(&parent) {
                self.nodes.insert(i, parent);
            }

            let existing_parents = self.parents.entry(node).or_default();

            if let Err(i) = existing_parents.binary_search(&parent) {
                existing_parents.insert(i, parent);
            }
        }

        self
    }

    fn build(&mut self) -> (Tangle, HashMap<usize, MessageId>) {
        // Check that the graph is a DAG and find a topological order so we can add messages to the
        // tangle in the correct order (parents before children). This `Vec` will hold the nodes in
        // such order.
        let mut ordered_nodes = vec![];

        // Tarjan's algorithm for topological sorting.
        fn visit(
            node: usize,
            perms: &mut Vec<usize>,
            temps: &mut Vec<usize>,
            parents: &HashMap<usize, Vec<usize>>,
            ordered_nodes: &mut Vec<usize>,
        ) {
            if perms.binary_search(&node).is_ok() {
                return;
            }

            match temps.binary_search(&node) {
                Ok(_) => panic!("not a DAG"),
                Err(i) => temps.insert(i, node),
            }

            if let Some(edges) = parents.get(&node) {
                for &edge in edges {
                    visit(edge, perms, temps, parents, ordered_nodes);
                }
            }

            match temps.binary_search(&node) {
                Ok(i) => {
                    temps.remove(i);
                    match perms.binary_search(&node) {
                        Ok(_) => unreachable!(),
                        Err(i) => perms.insert(i, node),
                    }
                }
                Err(_) => unreachable!(),
            }

            ordered_nodes.insert(0, node);
        }

        let mut perms = vec![];
        let mut temps = vec![];

        for &node in self.nodes.iter() {
            visit(node, &mut perms, &mut temps, &mut self.parents, &mut ordered_nodes);
        }

        let tangle = Tangle::new();
        let mut ids = HashMap::new();

        while let Some(node) = ordered_nodes.pop() {
            let (_msg, _meta, id) = match self.parents.get(&node) {
                Some(parents) if !parents.is_empty() => {
                    let parents = parents.iter().map(|node| ids[node]).collect::<Vec<MessageId>>();
                    new_node(&tangle, parents)
                }
                _ => new_sep(&tangle),
            };

            assert!(ids.insert(node, id).is_none());
        }

        (tangle, ids)
    }
}

#[test]
fn walk() {
    // 0 --
    //     | -- 8 --
    // 1 --         |
    //              | -- 12 --
    // 2 --         |          |
    //     | -- 9 --           |
    // 3 --                    |
    //                         | -- 14
    // 4 --                    |
    //     | -- 10 --          |
    // 5 --          |         |
    //               | -- 13 --
    // 6 --          |
    //     | -- 11 --
    // 7 --

    let (tangle, ids) = TangleBuilder::new()
        .add_node(0, [])
        .add_node(1, [])
        .add_node(2, [])
        .add_node(3, [])
        .add_node(4, [])
        .add_node(5, [])
        .add_node(6, [])
        .add_node(7, [])
        .add_node(8, [0, 1])
        .add_node(9, [2, 3])
        .add_node(10, [4, 5])
        .add_node(11, [6, 7])
        .add_node(12, [8, 9])
        .add_node(13, [10, 11])
        .add_node(14, [12, 13])
        .build();

    let correct_order = vec![
        ids[&14],
        ids[&12],
        ids[&8],
        ids[&0],
        ids[&1],
        ids[&9],
        ids[&2],
        ids[&3],
        ids[&13],
        ids[&10],
        ids[&4],
        ids[&5],
        ids[&11],
        ids[&6],
        ids[&7],
    ];

    let mut traversed_order = Vec::with_capacity(correct_order.len());

    let mut walker = TangleWalker::new(&tangle, ids[&14]);

    while let Some(status) = walker.next() {
        if let TangleWalkerStatus::Matched(message_id, message_data) = status {
            traversed_order.push(message_id);
        } else {
            println!("{:?}", status);
        }
    }

    // assert_eq!(correct_order, traversed_order);
}
