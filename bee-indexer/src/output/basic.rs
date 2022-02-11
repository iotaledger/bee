// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct Extended {
    pub output_id: String,
    pub amount: i64,
    pub sender: Option<String>,
    pub tag: Option<String>,
    pub address: Option<String>,
    pub milestone_index: String,
}
