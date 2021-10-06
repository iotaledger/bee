// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};
use bee_tangle::Tangle;
use bee_test::rand::{
    bytes::rand_bytes_array,
    message::{metadata::rand_message_metadata, rand_message_with_parents_ids},
};

use std::collections::HashMap;

fn rand_prefixed_message_id(prefix: u16) -> MessageId {
    let mut message_id_bytes = rand_bytes_array();

    message_id_bytes[0..2].copy_from_slice(&prefix.to_le_bytes());

    MessageId::from(message_id_bytes)
}

fn new_node(tangle: &Tangle, message_id: MessageId, parents_ids: Vec<MessageId>) -> (Message, MessageMetadata) {
    let message = rand_message_with_parents_ids(parents_ids);
    let metadata = rand_message_metadata();

    tangle.insert(message_id, message.clone(), metadata.clone());

    (message, metadata)
}

pub struct TangleBuilder {
    graph: HashMap<usize, Vec<usize>>,
}

impl TangleBuilder {
    pub fn new() -> Self {
        Self { graph: HashMap::new() }
    }

    pub fn add_node<const M: usize>(&mut self, node: usize, parents: [usize; M]) -> &mut Self {
        // Get the parents of the node or insert an empty list of parents if the node was not in the graph.
        let existing_parents = self.graph.entry(node).or_default();

        for parent in &parents {
            // Check if the parent is not in the list of parents of the node and insert it.
            if let Err(i) = existing_parents.binary_search(parent) {
                existing_parents.insert(i, *parent);
            }
        }

        self
    }

    pub fn build(self) -> (Tangle, HashMap<usize, MessageId>) {
        // Check that the graph is a DAG and find a topological order so we can add messages to the tangle in the
        // correct order (parents before children). This `Vec` will hold the nodes in such order.
        let mut ordered_nodes = vec![];

        // Tarjan's algorithm for topological sorting.
        fn visit(
            node: usize,
            perms: &mut Vec<usize>,
            temps: &mut Vec<usize>,
            parents: &HashMap<usize, Vec<usize>>,
            ordered_nodes: &mut Vec<usize>,
        ) {
            // If the node is permanently marked as visited we skip it.
            if perms.binary_search(&node).is_ok() {
                return;
            }

            // If the node is temporarily marked as visited we have a cycle.
            let i = temps.binary_search(&node).map(|i| temps[i]).expect_err("Found cycle");
            // Mark the node as temporarily visited.
            temps.insert(i, node);

            // Visit each parent of the node.
            if let Some(edges) = parents.get(&node) {
                for &edge in edges {
                    visit(edge, perms, temps, parents, ordered_nodes);
                }
            }

            // Remove the temporary mark for the node.
            let j = temps
                .binary_search(&node)
                .map_err(|j| temps[j])
                .expect("A temporarily marked node cannot be unmarked while visiting oter node");
            temps.remove(j);

            // Mark the node as permanently visited.
            let k = perms
                .binary_search(&node)
                .map(|k| perms[k])
                .expect_err("A temporarily marked node cannot be permanently marked");
            perms.insert(k, node);

            // Insert the node at the beginning of the list.
            ordered_nodes.insert(0, node);
        }

        let mut perms = vec![];
        let mut temps = vec![];
        let unmarked = self.graph.keys().copied().collect::<Vec<_>>();

        for node in unmarked {
            visit(node, &mut perms, &mut temps, &self.graph, &mut ordered_nodes);
        }

        let tangle = Tangle::new();
        let mut ids = HashMap::new();

        while let Some(node) = ordered_nodes.pop() {
            let id = rand_prefixed_message_id(node as u16);

            if let Some(parents) = self.graph.get(&node) {
                if parents.is_empty() {
                    new_node(&tangle, id, vec![MessageId::null()]);
                } else {
                    let parents = parents.iter().map(|node| ids[node]).collect::<Vec<MessageId>>();
                    new_node(&tangle, id, parents);
                }
            }

            assert!(ids.insert(node, id).is_none());
        }

        (tangle, ids)
    }
}
