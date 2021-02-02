// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::Payload, Error, MessageId,MESSAGE_ID_LENGTH};

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::vec::Vec;

pub const MILESTONE_MERKLE_PROOF_LENGTH: usize = 32;
pub const MILESTONE_PUBLIC_KEY_LENGTH: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MilestonePayloadEssence {
    index: u32,
    timestamp: u64,
    parents: Vec<MessageId>,
    merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
    public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
    receipt: Option<Payload>,
}

impl MilestonePayloadEssence {
    pub fn new(
        index: u32,
        timestamp: u64,
        parents: Vec<MessageId>,
        merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
        public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
    ) -> Self {
        Self {
            index,
            timestamp,
            parents,
            merkle_proof,
            public_keys,
            receipt: None,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn parents(&self) -> &[MessageId] {
        &self.parents
    }

    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }

    pub fn public_keys(&self) -> &Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]> {
        &self.public_keys
    }

    pub fn receipt(&self) -> Option<&Payload> {
        self.receipt.as_ref()
    }
}

impl Packable for MilestonePayloadEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.timestamp.packed_len()
            + 0u8.packed_len()
            + self.parents.len() * MESSAGE_ID_LENGTH
            + MILESTONE_MERKLE_PROOF_LENGTH
            + 0u8.packed_len()
            + self.public_keys.len() * MILESTONE_PUBLIC_KEY_LENGTH
            + 0u32.packed_len()
            + self.receipt.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;

        self.timestamp.pack(writer)?;

        (self.parents().len() as u8).pack(writer)?;

        for parent in self.parents().iter() {
            parent.pack(writer)?;
        }

        writer.write_all(&self.merkle_proof)?;

        (self.public_keys.len() as u8).pack(writer)?;

        for public_key in &self.public_keys {
            writer.write_all(public_key)?;
        }

        match self.receipt {
            Some(ref receipt) => {
                (receipt.packed_len() as u32).pack(writer)?;
                receipt.pack(writer)?;
            }
            None => 0u32.pack(writer)?,
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = u32::unpack(reader)?;

        let timestamp = u64::unpack(reader)?;

        let parents_len = u8::unpack(reader)? as usize;

        if parents_len != 2 {
            return Err(Error::InvalidParentsCount(parents_len));
        }

        let mut parents = Vec::with_capacity(parents_len);
        for _ in 0..parents_len {
            parents.push(MessageId::unpack(reader)?);
        }

        let mut merkle_proof = [0u8; MILESTONE_MERKLE_PROOF_LENGTH];
        reader.read_exact(&mut merkle_proof)?;

        let public_keys_len = u8::unpack(reader)? as usize;
        let mut public_keys = Vec::with_capacity(public_keys_len);
        for _ in 0..public_keys_len {
            let mut public_key = [0u8; MILESTONE_PUBLIC_KEY_LENGTH];
            reader.read_exact(&mut public_key)?;
            public_keys.push(public_key);
        }

        let receipt_len = u32::unpack(reader)? as usize;
        let receipt = if receipt_len > 0 {
            let receipt = Payload::unpack(reader)?;
            if receipt_len != receipt.packed_len() {
                return Err(Self::Error::InvalidAnnouncedLength(receipt_len, receipt.packed_len()));
            }
            if !matches!(receipt, Payload::Receipt(_)) {
                return Err(Error::InvalidPayloadKind(receipt.kind()));
            }
            Some(receipt)
        } else {
            None
        };

        // TODO builder ?

        Ok(Self {
            index,
            timestamp,
            parents,
            merkle_proof,
            public_keys,
            receipt,
        })
    }
}
