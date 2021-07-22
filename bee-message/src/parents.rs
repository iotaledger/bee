// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{MessageId, MessageUnpackError, ValidationError, MESSAGE_ID_LENGTH};

use bee_ord::is_unique_sorted;
use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryFrom, TryInto},
    ops::RangeInclusive,
};

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

/// Minimum number of strong parents for a valid message.
pub const MESSAGE_MIN_STRONG_PARENTS: usize = 1;

/// An individual message parent, which can be categorized as "strong" or "weak".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[repr(u8)]
pub enum ParentsType {
    /// Message parents in which the past cone is "Liked".
    Strong = 0,
    /// Message parents in which the past cone is "Disliked", but the parents themselves are "Liked".
    Weak = 1,
    /// Message parents that are "Liked".
    Disliked = 2,
    /// Message parents that are "Disliked".
    Liked = 3,
}

impl TryFrom<u8> for ParentsType {
    type Error = ValidationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Strong),
            1 => Ok(Self::Weak),
            2 => Ok(Self::Disliked),
            3 => Ok(Self::Liked),
            _ => Err(ValidationError::InvalidParentsType(value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
/// A block of message parent IDs, all of the same `ParentsType`.
///
/// `ParentsBlock`s must:
/// * Be of a valid `ParentsType`.
/// * Contain a valid count of parents (1..=8).
/// * IDs must be unique and lexicographically sorted in their serialized forms.
pub struct ParentsBlock {
    ty: ParentsType,
    ids: Vec<MessageId>,
}

impl ParentsBlock {
    /// Creates a new `ParentsBlock`, and validates the ID collection.
    pub fn new(ty: ParentsType, ids: Vec<MessageId>) -> Result<Self, ValidationError> {
        validate_parents_count(ids.len())?;
        validate_parents_unique_sorted(&ids)?;

        Ok(Self { ty, ids })
    }

    #[allow(clippy::len_without_is_empty)]
    /// Returns the number of `MessageIDs` in the `ParentsBlock` ID collection.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Returns an iterator over the `ParentsBlock` ID collection.
    pub fn iter(&self) -> impl Iterator<Item = &MessageId> {
        self.ids.iter()
    }
}

impl Packable for ParentsBlock {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + 0u8.packed_len() + self.ids.len() * MESSAGE_ID_LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        (self.ty as u8).pack(packer).map_err(PackError::infallible)?;
        (self.ids.len() as u8).pack(packer).map_err(PackError::infallible)?;

        for id in &self.ids {
            id.pack(packer).map_err(PackError::infallible)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let ty = u8::unpack(unpacker)
            .map_err(UnpackError::infallible)?
            .try_into()
            .map_err(|e: ValidationError| UnpackError::Packable(e.into()))?;

        let count = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_parents_count(count as usize).map_err(|e| UnpackError::Packable(e.into()))?;

        let mut ids = Vec::with_capacity(count as usize);
        for _ in 0..count as usize {
            ids.push(MessageId::unpack(unpacker).map_err(UnpackError::infallible)?);
        }

        validate_parents_unique_sorted(&ids).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { ty, ids })
    }
}

fn validate_parents_count(count: usize) -> Result<(), ValidationError> {
    if !MESSAGE_PARENTS_RANGE.contains(&count) {
        Err(ValidationError::InvalidParentsCount(count))
    } else {
        Ok(())
    }
}

fn validate_parents_unique_sorted(parents: &[MessageId]) -> Result<(), ValidationError> {
    if !is_unique_sorted(parents.iter()) {
        Err(ValidationError::ParentsNotUniqueSorted)
    } else {
        Ok(())
    }
}
