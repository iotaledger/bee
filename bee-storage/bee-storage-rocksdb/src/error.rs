// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("RocksDB error: {0}.")]
    RocksDB(#[from] rocksdb::Error),
    #[error("Unknown column family {0}.")]
    UnknownCf(&'static str),
}
