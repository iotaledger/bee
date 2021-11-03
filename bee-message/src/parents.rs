// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of message parents.

use crate::{MessageId, MessageUnpackError, ValidationError};

use bee_ord::is_unique_sorted;
use bee_packable::{coerce::*, Packable, Packer, UnpackError, Unpacker};

use bitvec::prelude::*;

use alloc::{vec, vec::Vec};
use core::{
    cmp,
    ops::{Deref, RangeInclusive},
};

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<usize> = 1..=8;

/// Minimum number of strong parents for a valid message.
pub const MESSAGE_STRONG_PARENTS_MIN: usize = 1;

/// An individual message parent, which can be categorized as "strong" or "weak".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Parent {
    Strong(MessageId),
    Weak(MessageId),
}

impl Parent {
    /// Returns the [`MessageId`] of this [`Parent`].
    pub fn id(&self) -> &MessageId {
        match self {
            Self::Strong(id) => id,
            Self::Weak(id) => id,
        }
    }
}

impl PartialOrd for Parent {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.id().cmp(other.id()))
    }
}

impl Ord for Parent {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// A [`Message`](crate::Message)'s `Parents` are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must:
/// * be in the [`MESSAGE_PARENTS_RANGE`] range;
/// * contain at least [`MESSAGE_STRONG_PARENTS_MIN`] strong parents;
/// * be lexicographically sorted;
/// * be unique;
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents(Vec<Parent>);

impl Deref for Parents {
    type Target = [Parent];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// Creates a new [`Parents`] collection.
    pub fn new(parents: Vec<Parent>) -> Result<Self, ValidationError> {
        validate_parents_count(parents.len())?;
        validate_unique_sorted(&parents)?;

        let strong_count = parents.iter().fold(0usize, |acc, parent| match parent {
            Parent::Strong(_) => acc + 1,
            _ => acc,
        });

        validate_strong_parents_count(strong_count)?;

        Ok(Self(parents))
    }

    /// Returns the number of parents.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the parents.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Parent> {
        self.0.iter()
    }
}

impl Packable for Parents {
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + 0u8.packed_len() + self.0.len() * MessageId::LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (self.len() as u8).pack(packer)?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];

        for (i, parent) in self.iter().enumerate() {
            let is_strong = matches!(parent, Parent::Strong(_));
            bits.set(i, is_strong);
        }

        let bits_repr = bits.load::<u8>();
        bits_repr.pack(packer)?;

        for id in self.iter().map(Parent::id) {
            id.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let count = u8::unpack(unpacker).infallible()?;
        validate_parents_count(count as usize).map_err(UnpackError::from_packable)?;

        let bits_repr = u8::unpack(unpacker).infallible()?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];
        bits.store(bits_repr);

        validate_strong_parents_count(bits.count_ones()).map_err(UnpackError::from_packable)?;

        let mut parents = vec![];
        parents.reserve(count as usize);

        for i in 0..count {
            let id = MessageId::unpack(unpacker).infallible()?;

            if *bits.get(i as usize).unwrap() {
                parents.push(Parent::Strong(id))
            } else {
                parents.push(Parent::Weak(id))
            }
        }

        validate_unique_sorted(&parents).map_err(UnpackError::from_packable)?;

        Ok(Self(parents))
    }
}

fn validate_parents_count(count: usize) -> Result<(), ValidationError> {
    if !MESSAGE_PARENTS_RANGE.contains(&count) {
        Err(ValidationError::InvalidParentsCount(count))
    } else {
        Ok(())
    }
}

fn validate_unique_sorted(parents: &[Parent]) -> Result<(), ValidationError> {
    if !is_unique_sorted(parents.iter().map(|parent| parent.id().as_ref())) {
        Err(ValidationError::ParentsNotUniqueSorted)
    } else {
        Ok(())
    }
}

fn validate_strong_parents_count(count: usize) -> Result<(), ValidationError> {
    if count < MESSAGE_STRONG_PARENTS_MIN {
        Err(ValidationError::InvalidStrongParentsCount(count))
    } else {
        Ok(())
    }
}
