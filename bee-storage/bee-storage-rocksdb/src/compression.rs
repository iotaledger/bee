// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rocksdb::DBCompressionType;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Snappy,
    Zlib,
    Bz2,
    Lz4,
    Lz4hc,
    Zstd,
}

impl From<CompressionType> for DBCompressionType {
    fn from(compression: CompressionType) -> Self {
        match compression {
            CompressionType::None => DBCompressionType::None,
            CompressionType::Snappy => DBCompressionType::Snappy,
            CompressionType::Zlib => DBCompressionType::Zlib,
            CompressionType::Bz2 => DBCompressionType::Bz2,
            CompressionType::Lz4 => DBCompressionType::Lz4,
            CompressionType::Lz4hc => DBCompressionType::Lz4hc,
            CompressionType::Zstd => DBCompressionType::Zstd,
        }
    }
}
