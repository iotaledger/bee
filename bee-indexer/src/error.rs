// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use sea_orm::error::DbErr;

#[derive(Debug)]
pub enum IndexerError {
    InvalidCursorLength(usize),
    OffsetParseError(std::array::TryFromSliceError),
    DatabaseError(DbErr),
}
