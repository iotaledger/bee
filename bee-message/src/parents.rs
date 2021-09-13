// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of message parents.

use crate::{MessageId, MessageUnpackError, ValidationError};

use bee_ord::is_unique_sorted;
use bee_packable::{
    coerce::{PackCoerceInfallible, UnpackCoerceInfallible},
    BoundedU8, InvalidBoundedU8, PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use bitvec::prelude::*;

use alloc::{vec, vec::Vec};
use core::{
    convert::{Infallible, TryInto},
    ops::Deref,
};

/// Minimum number of parents for a valid message.
pub const PREFIXED_PARENTS_LENGTH_MIN: u8 = 1;
/// Maximum number of parents for a valid message.
pub const PREFIXED_PARENTS_LENGTH_MAX: u8 = 8;

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

/// A [`Message`](crate::Message)'s `Parents` are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must be:
/// * in the `MESSAGE_PARENTS_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents {
    inner: VecPrefix<Parent, BoundedU8<PREFIXED_PARENTS_LENGTH_MIN, PREFIXED_PARENTS_LENGTH_MAX>>,
}

impl Deref for Parents {
    type Target = [Parent];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// Creates a new [`Parents`] collection.
    pub fn new(inner: Vec<Parent>) -> Result<Self, ValidationError> {
        if !is_unique_sorted(inner.iter().map(|parent| parent.id().as_ref())) {
            return Err(ValidationError::ParentsNotUniqueSorted);
        }

        let strong_count = inner.iter().fold(0usize, |acc, parent| match parent {
            Parent::Strong(_) => acc + 1,
            _ => acc,
        });

        validate_strong_parents_count(strong_count)?;

        let prefixed = inner.try_into().map_err(
            |err: InvalidBoundedU8<PREFIXED_PARENTS_LENGTH_MIN, PREFIXED_PARENTS_LENGTH_MAX>| {
                ValidationError::InvalidParentsCount(err.0 as usize)
            },
        )?;

        Ok(Self { inner: prefixed })
    }

    /// Returns the number of parents.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns an iterator over the parents.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Parent> {
        self.inner.iter()
    }
}

impl Packable for Parents {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len() + self.inner.len() * MessageId::LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        (self.len() as u8).pack(packer).infallible()?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];

        for (i, parent) in self.iter().enumerate() {
            let is_strong = matches!(parent, Parent::Strong(_));
            bits.set(i, is_strong);
        }

        let bits_repr = bits.load::<u8>();
        bits_repr.pack(packer).infallible()?;

        for id in self.iter().map(Parent::id) {
            id.pack(packer).infallible()?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let count = u8::unpack(unpacker).infallible()?;
        let bits_repr = u8::unpack(unpacker).infallible()?;

        let mut bits = bitarr![Lsb0, u8; 0; 8];
        bits.store(bits_repr);

        validate_strong_parents_count(bits.count_ones()).map_err(|e| UnpackError::Packable(e.into()))?;

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

        let prefixed = parents.try_into().map_err(
            |err: InvalidBoundedU8<PREFIXED_PARENTS_LENGTH_MIN, PREFIXED_PARENTS_LENGTH_MAX>| {
                UnpackError::Packable(MessageUnpackError::Validation(ValidationError::InvalidParentsCount(
                    err.0 as usize,
                )))
            },
        )?;

        Ok(Self { inner: prefixed })
    }
}

fn validate_strong_parents_count(count: usize) -> Result<(), ValidationError> {
    if count < MESSAGE_MIN_STRONG_PARENTS {
        Err(ValidationError::InvalidStrongParentsCount(count))
    } else {
        Ok(())
    }
}
