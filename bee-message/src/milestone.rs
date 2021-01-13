// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageId;

use bee_common::packable::{Packable, Read, Write};

use serde::Deserialize;
use thiserror::Error;

use std::ops::{Add, Deref, Sub};

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error {0}")]
    IO(#[from] std::io::Error),
    #[error("MessageId error {0}")]
    MessageId(<MessageId as Packable>::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub(crate) message_id: MessageId,
    pub(crate) timestamp: u64,
}

impl Milestone {
    pub fn new(message_id: MessageId, timestamp: u64) -> Self {
        Self { message_id, timestamp }
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl Packable for Milestone {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.message_id.packed_len() + self.timestamp.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.message_id.pack(writer).map_err(Error::MessageId)?;
        self.timestamp.pack(writer).map_err(Error::IO)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            message_id: MessageId::unpack(reader).map_err(Error::MessageId)?,
            timestamp: u64::unpack(reader)?,
        })
    }
}

/// A wrapper around a `u32` that represents a milestone index.
#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
pub struct MilestoneIndex(pub u32);

impl Deref for MilestoneIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for MilestoneIndex {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl Add for MilestoneIndex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(*self + *other)
    }
}

impl Sub for MilestoneIndex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(*self - *other)
    }
}

impl Packable for MilestoneIndex {
    type Error = std::io::Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self(u32::unpack(reader)?))
    }
}
