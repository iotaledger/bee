// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputsResponse {
    pub ledger_index: u32,
    pub items: Vec<String>,
    pub cursor: Option<String>,
}
