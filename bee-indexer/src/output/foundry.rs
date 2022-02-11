// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct Foundry {
    pub foundry_id: String,
    pub output_id: String,
    pub amount: i64,
    pub address: Option<String>,
    pub milestone_index: String,
}
