// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct Nft {
    pub nft_id: String,
    pub output_id: String,
    pub amount: i64,
    pub issuer: Option<String>,
    pub sender: Option<String>,
    pub tag: Option<String>,
    pub address: Option<String>,
    pub milestone_index: String,
}
