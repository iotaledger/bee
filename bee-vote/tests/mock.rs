// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_vote::{
    error::Error,
    opinion::{OpinionGiver, Opinions, QueryIds},
};

use bee_message::{{payload::transaction::TransactionId}, MessageId};
use bee_network::PeerId;

use rand::{distributions::Alphanumeric, thread_rng, Rng};

/// Mock opinion giver struct for instantiation in testing.
#[derive(Clone)]
pub(crate) struct MockOpinionGiver {
    pub(crate) id: String,
    pub(crate) round: u32,
    pub(crate) round_replies: Vec<Opinions>,
}

impl OpinionGiver for MockOpinionGiver {
    fn query(&mut self, _: &QueryIds) -> Result<Opinions, Error> {
        if self.round as usize >= self.round_replies.len() {
            return Ok(self.round_replies.last().unwrap().clone());
        }

        let opinions = self.round_replies.get(self.round as usize).unwrap().clone();
        self.round += 1;

        Ok(opinions)
    }

    fn id(&self) -> &str {
        &self.id
    }
}

pub(crate) fn rand_message_id() -> MessageId {
    MessageId::new(random_id_bytes())
}

pub(crate) fn rand_transaction_id() -> TransactionId {
    TransactionId::new(random_id_bytes())
}

pub(crate) fn rand_node_id() -> PeerId {
    PeerId::random()
}

pub(crate) fn random_id_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

fn random_id_bytes() -> [u8; 32] {
    thread_rng().gen::<[u8; 32]>()
}
