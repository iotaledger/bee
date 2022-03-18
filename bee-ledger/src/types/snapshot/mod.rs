// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing types to describe and handle ledger snapshots.

/// Module containing types to describe snapshot headers.
pub mod header;
/// Module containing a type to describe snapshot information.
pub mod info;
/// Module containing a snapshot kind enumeration.
pub mod kind;
/// Module containing a type to describe the ledger changes occurring within a milestone.
pub mod milestone_diff;

pub use self::header::{DeltaSnapshotHeader, FullSnapshotHeader, SnapshotHeader};
pub use self::info::SnapshotInfo;
pub use self::kind::SnapshotKind;
pub use self::milestone_diff::MilestoneDiff;
