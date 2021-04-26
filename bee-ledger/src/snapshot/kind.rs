// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::snapshot::error::Error;

use bee_common::packable::{Packable, Read, Write};

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Kind {
    /// Full is a snapshot which contains the full ledger entry for a given milestone plus the milestone diffs which
    /// subtracted to the ledger milestone reduce to the snapshot milestone ledger.
    Full = 0,
    /// Delta is a snapshot which contains solely diffs of milestones newer than a certain ledger milestone instead of
    /// the complete ledger state of a given milestone.
    Delta = 1,
}

impl Packable for Kind {
    type Error = Error;

    fn packed_len(&self) -> usize {
        (*self as u8).packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (*self as u8).pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            0 => Kind::Full,
            1 => Kind::Delta,
            k => return Err(Self::Error::InvalidKind(k)),
        })
    }
}
