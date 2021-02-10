// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::Payload, Error, MessageId, MESSAGE_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};
use bee_pow::providers::{Miner, Provider, ProviderBuilder};

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use serde::{Deserialize, Serialize};

use std::{
    ops::RangeInclusive,
    sync::{atomic::AtomicBool, Arc},
};

pub const MESSAGE_LENGTH_MAX: usize = 32768;
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    network_id: u64,
    parents: Vec<MessageId>,
    payload: Option<Payload>,
    nonce: u64,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    pub fn id(&self) -> (MessageId, Vec<u8>) {
        let mut hasher = VarBlake2b::new(MESSAGE_ID_LENGTH).unwrap();
        let bytes = self.pack_new();

        hasher.update(&bytes);

        let mut id = [0u8; MESSAGE_ID_LENGTH];
        hasher.finalize_variable(|res| id.copy_from_slice(res));

        (MessageId::new(id), bytes)
    }

    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    pub fn parents(&self) -> &[MessageId] {
        &self.parents
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Packable for Message {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.network_id.packed_len()
            + 0u8.packed_len()
            + self.parents.len() * MESSAGE_ID_LENGTH
            + 0u32.packed_len()
            + self.payload.as_ref().map_or(0, Packable::packed_len)
            + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.network_id.pack(writer)?;

        (self.parents().len() as u8).pack(writer)?;

        for parent in self.parents().iter() {
            parent.pack(writer)?;
        }

        if let Some(ref payload) = self.payload {
            (payload.packed_len() as u32).pack(writer)?;
            payload.pack(writer)?;
        } else {
            0u32.pack(writer)?;
        }

        self.nonce.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let network_id = u64::unpack(reader)?;

        let parents_len = u8::unpack(reader)? as usize;

        if !MESSAGE_PARENTS_RANGE.contains(&parents_len) {
            return Err(Error::InvalidParentsCount(parents_len));
        }

        let mut parents = Vec::with_capacity(parents_len);
        for _ in 0..parents_len {
            parents.push(MessageId::unpack(reader)?);
        }

        let payload_len = u32::unpack(reader)? as usize;
        let payload = if payload_len != 0 {
            let payload = Payload::unpack(reader)?;
            if payload_len != payload.packed_len() {
                return Err(Self::Error::InvalidAnnouncedLength(payload_len, payload.packed_len()));
            }
            Some(payload)
        } else {
            None
        };

        let nonce = u64::unpack(reader)?;

        // Computed instead of calling `packed_len` on Self because `payload_len` is already known and it may be
        // expensive to call `payload.packed_len()` twice.
        let message_len =
            network_id.packed_len() + parents.len() * MESSAGE_ID_LENGTH + payload_len + nonce.packed_len();

        if message_len > MESSAGE_LENGTH_MAX {
            return Err(Error::InvalidMessageLength(message_len));
        }

        Ok(Self {
            network_id,
            parents,
            payload,
            nonce,
        })
    }
}

pub struct MessageBuilder<P: Provider = Miner> {
    network_id: Option<u64>,
    parents: Option<Vec<MessageId>>,
    payload: Option<Payload>,
    nonce_provider: Option<(P, f64, Option<Arc<AtomicBool>>)>,
}

impl<P: Provider> Default for MessageBuilder<P> {
    fn default() -> Self {
        Self {
            network_id: None,
            parents: None,
            payload: None,
            nonce_provider: None,
        }
    }
}

impl<P: Provider> MessageBuilder<P> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_network_id(mut self, network_id: u64) -> Self {
        self.network_id = Some(network_id);
        self
    }

    pub fn with_parents(mut self, parents: Vec<MessageId>) -> Self {
        self.parents = Some(parents);
        self
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_nonce_provider(mut self, nonce_provider: P, target_score: f64, done: Option<Arc<AtomicBool>>) -> Self {
        self.nonce_provider = Some((nonce_provider, target_score, done));
        self
    }

    pub fn finish(self) -> Result<Message, Error> {
        let mut message = Message {
            network_id: self.network_id.ok_or(Error::MissingField("network_id"))?,
            parents: self.parents.ok_or(Error::MissingField("parents"))?,
            payload: self.payload,
            nonce: 0,
        };

        if !MESSAGE_PARENTS_RANGE.contains(&message.parents.len()) {
            return Err(Error::InvalidParentsCount(message.parents.len()));
        }

        message.parents.sort_unstable();
        message.parents.dedup();

        let message_bytes = message.pack_new();

        if message_bytes.len() > MESSAGE_LENGTH_MAX {
            return Err(Error::InvalidMessageLength(message_bytes.len()));
        }

        let (nonce_provider, target_score, done) =
            self.nonce_provider
                .unwrap_or((P::Builder::new().finish(), 4000f64, None));

        message.nonce = nonce_provider
            .nonce(
                &message_bytes[..message_bytes.len() - std::mem::size_of::<u64>()],
                target_score,
                done,
            )
            .unwrap_or(0);

        Ok(message)
    }
}
