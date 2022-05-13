// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The parents module defines the core data type for storing the blocks directly approved by a block.

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use derive_more::Deref;
use iterator_sorted::is_unique_sorted;
use packable::{bounded::BoundedU8, prefix::BoxedSlicePrefix, Packable, PackableExt};

use crate::{BlockId, Error};

pub(crate) type ParentCount = BoundedU8<{ *Parents::COUNT_RANGE.start() }, { *Parents::COUNT_RANGE.end() }>;

/// A [`Block`](crate::Block)'s [`Parents`] are the [`BlockId`]s of the blocks it directly approves.
///
/// Parents must be:
/// * in the `Parents::COUNT_RANGE` range;
/// * lexicographically sorted;
/// * unique;
#[derive(Clone, Debug, Eq, PartialEq, Deref, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[deref(forward)]
#[packable(unpack_error = Error, with = |e| Error::InvalidParentCount(e.into_prefix_err().into()))]
pub struct Parents(#[packable(verify_with = verify_parents)] BoxedSlicePrefix<BlockId, ParentCount>);

#[allow(clippy::len_without_is_empty)]
impl Parents {
    /// The range representing the valid number of parents.
    pub const COUNT_RANGE: RangeInclusive<u8> = 1..=8;

    /// Creates new [`Parents`].
    pub fn new(mut inner: Vec<BlockId>) -> Result<Self, Error> {
        inner.sort_unstable_by_key(|a| a.pack_to_vec());
        inner.dedup();

        Ok(Self(
            inner.into_boxed_slice().try_into().map_err(Error::InvalidParentCount)?,
        ))
    }

    /// Returns the number of parents.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the parents.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &BlockId> + '_ {
        self.0.iter()
    }
}

fn verify_parents<const VERIFY: bool>(parents: &[BlockId]) -> Result<(), Error> {
    if VERIFY && !is_unique_sorted(parents.iter().map(AsRef::as_ref)) {
        Err(Error::ParentsNotUniqueSorted)
    } else {
        Ok(())
    }
}
