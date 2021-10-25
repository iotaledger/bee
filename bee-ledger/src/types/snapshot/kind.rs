// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::Error;

use bee_packable::Packable;

/// The kind of a snapshot.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Packable)]
#[packable(tag_type = u8, with_error = Error::InvalidSnapshotKind)]
#[packable(unpack_error = Error)]
pub enum SnapshotKind {
    /// Full is a snapshot which contains the full ledger entry for a given milestone plus the milestone diffs which
    /// subtracted to the ledger milestone reduce to the snapshot milestone ledger.
    Full = 0,
    /// Delta is a snapshot which contains solely diffs of milestones newer than a certain ledger milestone instead of
    /// the complete ledger state of a given milestone.
    Delta = 1,
}
