// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rocksdb::DBCompressionType;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum CompressionType {
    Bz2,
    Lz4,
    Lz4hc,
    None,
    Snappy,
    Zlib,
    Zstd,
}

impl From<CompressionType> for DBCompressionType {
    fn from(compression: CompressionType) -> Self {
        match compression {
            CompressionType::Bz2 => DBCompressionType::Bz2,
            CompressionType::Lz4 => DBCompressionType::Lz4,
            CompressionType::Lz4hc => DBCompressionType::Lz4hc,
            CompressionType::None => DBCompressionType::None,
            CompressionType::Snappy => DBCompressionType::Snappy,
            CompressionType::Zlib => DBCompressionType::Zlib,
            CompressionType::Zstd => DBCompressionType::Zstd,
        }
    }
}
