// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of message parents.

use crate::{MessageId, MessageUnpackError, ValidationError};

use bee_ord::is_unique_sorted;
use bee_packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::VecPrefix,
    unpacker::Unpacker,
    Packable,
};

use bitvec::prelude::*;

use alloc::{vec, vec::Vec};
use core::{
    cmp,
    ops::{Deref, RangeInclusive},
};

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<u8> = MESSAGE_PARENTS_MIN..=MESSAGE_PARENTS_MAX;
/// The minimum valid number of parents.
pub const MESSAGE_PARENTS_MIN: u8 = 1;
/// The maximum valid number of parents.
pub const MESSAGE_PARENTS_MAX: u8 = 8;

/// Minimum number of strong parents for a valid message.
pub const MESSAGE_STRONG_PARENTS_MIN: usize = 1;

/// A valid parent count.
pub type ParentsCount = BoundedU8<MESSAGE_PARENTS_MIN, MESSAGE_PARENTS_MAX>;

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
pub struct Parents(VecPrefix<Parent, ParentsCount>);

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
        let parents: VecPrefix<_, ParentsCount> = parents.try_into().map_err(ValidationError::InvalidParentsCount)?;

        validate_unique_sorted(&parents)?;

        let strong_count = parents
            .iter()
            .filter(|parent| matches!(parent, Parent::Strong(_)))
            .count();

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

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let count: usize = ParentsCount::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| MessageUnpackError::Validation(ValidationError::InvalidParentsCount(err.into())))?
            .get()
            .into();

        let bits_repr = u8::unpack::<_, VERIFY>(unpacker).infallible()?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];
        bits.store(bits_repr);

        validate_strong_parents_count(bits.count_ones()).map_err(UnpackError::from_packable)?;

        let mut parents = vec![];
        parents.reserve(count);

        for i in 0..count {
            let id = MessageId::unpack::<_, VERIFY>(unpacker).infallible()?;

            if *bits.get(i).unwrap() {
                parents.push(Parent::Strong(id))
            } else {
                parents.push(Parent::Weak(id))
            }
        }

        validate_unique_sorted(&parents).map_err(UnpackError::from_packable)?;

        // `count` was already inbounds.
        Ok(Self(parents.try_into().unwrap()))
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
