// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, MessageId};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::vec::Vec;

pub const MILESTONE_MERKLE_PROOF_LENGTH: usize = 32;
pub const MILESTONE_PUBLIC_KEY_LENGTH: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MilestonePayloadEssence {
    index: u32,
    timestamp: u64,
    parent1: MessageId,
    parent2: MessageId,
    merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
    public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
}

impl MilestonePayloadEssence {
    pub fn new(
        index: u32,
        timestamp: u64,
        parent1: MessageId,
        parent2: MessageId,
        merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
        public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
    ) -> Self {
        Self {
            index,
            timestamp,
            parent1,
            parent2,
            merkle_proof,
            public_keys,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn parent1(&self) -> &MessageId {
        &self.parent1
    }

    pub fn parent2(&self) -> &MessageId {
        &self.parent2
    }

    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }

    pub fn public_keys(&self) -> &Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]> {
        &self.public_keys
    }
}

impl Packable for MilestonePayloadEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.timestamp.packed_len()
            + self.parent1.packed_len()
            + self.parent2.packed_len()
            + MILESTONE_MERKLE_PROOF_LENGTH
            + 0u8.packed_len()
            + self.public_keys.len() * MILESTONE_PUBLIC_KEY_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;

        self.timestamp.pack(writer)?;

        self.parent1.pack(writer)?;
        self.parent2.pack(writer)?;

        writer.write_all(&self.merkle_proof)?;

        (self.public_keys.len() as u8).pack(writer)?;

        for public_key in &self.public_keys {
            writer.write_all(public_key)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = u32::unpack(reader)?;

        let timestamp = u64::unpack(reader)?;

        let parent1 = MessageId::unpack(reader)?;
        let parent2 = MessageId::unpack(reader)?;

        let mut merkle_proof = [0u8; MILESTONE_MERKLE_PROOF_LENGTH];
        reader.read_exact(&mut merkle_proof)?;

        let public_keys_len = u8::unpack(reader)? as usize;
        let mut public_keys = Vec::with_capacity(public_keys_len);
        for _ in 0..public_keys_len {
            let mut public_key = [0u8; MILESTONE_PUBLIC_KEY_LENGTH];
            reader.read_exact(&mut public_key)?;
            public_keys.push(public_key);
        }

        Ok(Self {
            index,
            timestamp,
            parent1,
            parent2,
            merkle_proof,
            public_keys,
        })
    }
}
