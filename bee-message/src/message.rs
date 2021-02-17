// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::Payload, Error, MessageId, Parents, MESSAGE_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};
use bee_pow::providers::{Miner, Provider, ProviderBuilder};

use crypto::hashes::{blake2b::Blake2b256, Digest};

use std::sync::{atomic::AtomicBool, Arc};

pub const MESSAGE_LENGTH_MIN: usize = 53;
pub const MESSAGE_LENGTH_MAX: usize = 32768;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    network_id: u64,
    parents: Parents,
    payload: Option<Payload>,
    nonce: u64,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    // TODO should not return bytes anymore ?
    pub fn id(&self) -> (MessageId, Vec<u8>) {
        let bytes = self.pack_new();
        let id = Blake2b256::digest(&bytes);

        (MessageId::new(id.into()), bytes)
    }

    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    pub fn parents(&self) -> impl Iterator<Item = &MessageId> + '_ {
        self.parents.iter()
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
            + self.parents.packed_len()
            + 0u32.packed_len()
            + self.payload.as_ref().map_or(0, Packable::packed_len)
            + 0u64.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.network_id.pack(writer)?;

        self.parents.pack(writer)?;

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

        let parents = Parents::unpack(reader)?;

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
    parents: Option<Parents>,
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

    pub fn with_parents(mut self, parents: Parents) -> Self {
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

        // TODO move to Parents type
        // message.parents.sort_unstable();
        // message.parents.dedup();

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
