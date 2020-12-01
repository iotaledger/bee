// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rocksdb::DBCompactionStyle;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum CompactionStyle {
    Level,
    Universal,
    Fifo,
}

impl From<CompactionStyle> for DBCompactionStyle {
    fn from(compaction_style: CompactionStyle) -> Self {
        match compaction_style {
            CompactionStyle::Level => DBCompactionStyle::Level,
            CompactionStyle::Universal => DBCompactionStyle::Universal,
            CompactionStyle::Fifo => DBCompactionStyle::Fifo,
        }
    }
}
