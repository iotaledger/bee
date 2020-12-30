// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/src/plugins/dashboard/frontend/build/"]
pub(crate) struct Asset;
