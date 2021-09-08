// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rocksdb::DBCompactionStyle;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum CompactionStyle {
    Fifo,
    Level,
    Universal,
}

impl From<CompactionStyle> for DBCompactionStyle {
    fn from(compaction: CompactionStyle) -> Self {
        match compaction {
            CompactionStyle::Fifo => DBCompactionStyle::Fifo,
            CompactionStyle::Level => DBCompactionStyle::Level,
            CompactionStyle::Universal => DBCompactionStyle::Universal,
        }
    }
}
