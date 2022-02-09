// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/frontend/build/"]
pub(crate) struct Asset;
