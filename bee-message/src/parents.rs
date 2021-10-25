// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{Error, MessageId, MESSAGE_ID_LENGTH};

use bee_common::ord::is_unique_sorted;
use bee_packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::{TryIntoPrefixError, VecPrefix},
    unpacker::Unpacker,
    Packable,
};

use core::ops::{Deref, RangeInclusive};
use std::convert::Infallible;

/// The range representing the valid number of parents.
pub const MESSAGE_PARENTS_RANGE: RangeInclusive<u8> = MESSAGE_PARENTS_MIN..=MESSAGE_PARENTS_MAX;
/// The minimum number of parents.
pub const MESSAGE_PARENTS_MIN: u8 = 1;
/// The maximum number of parents.
pub const MESSAGE_PARENTS_MAX: u8 = 8;

/// A [`Message`](crate::Message)'s `Parents` are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must be:
/// * in the `MESSAGE_PARENTS_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Parents(VecPrefix<MessageId, BoundedU8<MESSAGE_PARENTS_MIN, MESSAGE_PARENTS_MAX>>);

impl Deref for Parents {
    type Target = [MessageId];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// Creates new `Parents`.
    pub fn new(inner: Vec<MessageId>) -> Result<Self, Error> {
        let inner = VecPrefix::<MessageId, BoundedU8<MESSAGE_PARENTS_MIN, MESSAGE_PARENTS_MAX>>::try_from(inner)
            .map_err(Error::InvalidParentsCount)?;

        if !is_unique_sorted(inner.iter().map(AsRef::as_ref)) {
            return Err(Error::ParentsNotUniqueSorted);
        }

        Ok(Self(inner))
    }

    /// Returns the number of parents.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the parents.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &MessageId> + '_ {
        self.0.iter()
    }
}

impl Packable for Parents {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (self.len() as u8).pack(packer)?;

        for parent in self.iter() {
            parent.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let parents_len: usize = BoundedU8::<MESSAGE_PARENTS_MIN, MESSAGE_PARENTS_MAX>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::InvalidParentsCount(TryIntoPrefixError::Invalid(err)))?
            .get()
            .into();

        let mut inner = Vec::with_capacity(parents_len);
        for _ in 0..parents_len {
            inner.push(MessageId::unpack::<_, VERIFY>(unpacker).infallible()?);
        }

        Self::new(inner).map_err(UnpackError::Packable)
    }
}
