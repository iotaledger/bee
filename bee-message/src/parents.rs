// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{MessageId, MessagePackError, MessageUnpackError, ValidationError, MESSAGE_ID_LENGTH};

use bee_ord::is_unique_sorted;
use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use bitvec::prelude::*;

use alloc::{vec, vec::Vec};
use core::ops::{Deref, RangeInclusive};

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

/// Minimum number of strong parents for a valid message.
pub const MESSAGE_MIN_STRONG_PARENTS: usize = 1;

/// An individual message parent, which can be categorized as "strong" or "weak".
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Parent {
    /// Message parents in which the past cone is "Liked".
    Strong(MessageId),
    /// Message parents in which the past cone is "Disliked", but the parents themselves are "Liked".
    Weak(MessageId),
}

impl Parent {
    /// Returns the `MessageId` of this `Parent`.
    pub fn id(&self) -> &MessageId {
        match self {
            Self::Strong(id) => id,
            Self::Weak(id) => id,
        }
    }
}

/// A `Message`'s `Parents` are the `MessageId`s of the messages it directly approves.
///
/// `Parents` must:
/// * Have length within the `MESSAGE_PARENTS_RANGE` range;
/// * Contain `MESSAGE_MIN_STRONG_PARENTS` strong `Parent`s;
/// * Be lexicographically sorted;
/// * Be unique;
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents {
    inner: Vec<Parent>,
}

impl Deref for Parents {
    type Target = [Parent];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// Creates a new `Parents` instance from a given collection.
    pub fn new(inner: Vec<Parent>) -> Result<Self, ValidationError> {
        validate_parents_count(inner.len())?;
        validate_parents_unique_sorted(&inner)?;

        let strong_count = inner.iter().fold(0usize, |acc, parent| match parent {
            Parent::Strong(_) => acc + 1,
            _ => acc,
        });

        validate_strong_parents_count(strong_count)?;

        Ok(Self { inner })
    }

    /// Returns the number of message parents.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns an `Iterator` over the strong parents of a message.
    pub fn strong_iter(&self) -> impl Iterator<Item = &MessageId> {
        self.inner
            .iter()
            .filter(|parent| matches!(parent, Parent::Strong(_)))
            .map(Parent::id)
    }

    /// Returns an `Iterator` over the weak parents of a message.
    pub fn weak_iter(&self) -> impl Iterator<Item = &MessageId> {
        self.inner
            .iter()
            .filter(|parent| matches!(parent, Parent::Weak(_)))
            .map(Parent::id)
    }
}

impl Packable for Parents {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + 0u8.packed_len() + self.inner.len() * MESSAGE_ID_LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        (self.len() as u8).pack(packer).map_err(PackError::infallible)?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];

        for (i, parent) in self.iter().enumerate() {
            let is_strong = matches!(parent, Parent::Strong(_));
            bits.set(i, is_strong);
        }

        let bits_repr = bits.load::<u8>();
        bits_repr.pack(packer).map_err(PackError::infallible)?;

        for id in self.iter().map(Parent::id) {
            id.pack(packer).map_err(PackError::infallible)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let count = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_parents_count(count as usize).map_err(|e| UnpackError::Packable(e.into()))?;

        let bits_repr = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];
        bits.store(bits_repr);
        validate_strong_parents_count(bits.count_ones()).map_err(|e| UnpackError::Packable(e.into()))?;

        let mut parents = vec![];
        parents.reserve(count as usize);

        for i in 0..count {
            let id = MessageId::unpack(unpacker).map_err(UnpackError::infallible)?;

            // Unwrap is fine here, since `i` has already been validated to be in `MESSAGE_PARENTS_RANGE`.
            if *bits.get(i as usize).unwrap() {
                parents.push(Parent::Strong(id))
            } else {
                parents.push(Parent::Weak(id))
            }
        }

        validate_parents_unique_sorted(&parents).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { inner: parents })
    }
}

fn validate_parents_count(count: usize) -> Result<(), ValidationError> {
    if !MESSAGE_PARENTS_RANGE.contains(&count) {
        Err(ValidationError::InvalidParentsCount(count))
    } else {
        Ok(())
    }
}

fn validate_strong_parents_count(count: usize) -> Result<(), ValidationError> {
    if count < MESSAGE_MIN_STRONG_PARENTS {
        Err(ValidationError::InvalidStrongParentsCount(count))
    } else {
        Ok(())
    }
}

fn validate_parents_unique_sorted(parents: &[Parent]) -> Result<(), ValidationError> {
    if !is_unique_sorted(parents.iter().map(|parent| parent.id().as_ref())) {
        Err(ValidationError::ParentsNotUniqueSorted)
    } else {
        Ok(())
    }
}
