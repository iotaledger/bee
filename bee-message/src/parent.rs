// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the messages directly approved by a message.

use crate::{Error, MessageId};

use derive_more::Deref;
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable};

use alloc::vec::Vec;
use core::ops::RangeInclusive;

pub(crate) type ParentCount = BoundedU8<{ *Parents::COUNT_RANGE.start() }, { *Parents::COUNT_RANGE.end() }>;

/// A [`Message`](crate::Message)'s [`Parents`] are the [`MessageId`]s of the messages it directly approves.
///
/// Parents must be:
/// * in the `Parents::COUNT_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq, Deref, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[deref(forward)]
#[packable(unpack_error = Error, with = |e| Error::InvalidParentCount(e.into_prefix().into()))]
pub struct Parents(#[packable(verify_with = verify_parents)] BoxedSlicePrefix<MessageId, ParentCount>);

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// The range representing the valid number of parents.
    pub const COUNT_RANGE: RangeInclusive<u8> = 1..=8;

    /// Creates new [`Parents`].
    pub fn new(inner: Vec<MessageId>) -> Result<Self, Error> {
        let inner: BoxedSlicePrefix<MessageId, ParentCount> =
            inner.into_boxed_slice().try_into().map_err(Error::InvalidParentCount)?;

        verify_parents::<true>(&inner)?;

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

fn verify_parents<const VERIFY: bool>(parents: &[MessageId]) -> Result<(), Error> {
    if VERIFY && !is_unique_sorted(parents.iter().map(AsRef::as_ref)) {
        Err(Error::ParentsNotUniqueSorted)
    } else {
        Ok(())
    }
}
