// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::Payload, Error, MessageId, Parents};

use bee_common::packable::{Packable, Read, Write};

use crypto::blake2b;

use alloc::vec::Vec;

pub const MILESTONE_MERKLE_PROOF_LENGTH: usize = 32;
pub const MILESTONE_PUBLIC_KEY_LENGTH: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestonePayloadEssence {
    index: u32,
    timestamp: u64,
    parents: Parents,
    merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
    public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
    receipt: Option<Payload>,
}

impl MilestonePayloadEssence {
    pub fn new(
        index: u32,
        timestamp: u64,
        parents: Parents,
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

    pub fn parents(&self) -> impl Iterator<Item = &MessageId> + '_ {
        self.parents.iter()
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

    pub fn hash(&self) -> [u8; 32] {
        let mut hash = [0u8; 32];

        blake2b::hash(&self.pack_new(), &mut hash);

        hash
    }
}

impl Packable for MilestonePayloadEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.timestamp.packed_len()
            + self.parents.packed_len()
            + MILESTONE_MERKLE_PROOF_LENGTH
            + 0u8.packed_len()
            + self.public_keys.len() * MILESTONE_PUBLIC_KEY_LENGTH
            + 0u32.packed_len()
            + self.receipt.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;

        self.timestamp.pack(writer)?;

        self.parents.pack(writer)?;

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

        let parents = Parents::unpack(reader)?;

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
