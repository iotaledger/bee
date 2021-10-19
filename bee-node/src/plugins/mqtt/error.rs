// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use librumqttd;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("mqtt operation failed: {0}")]
    Error(#[from] librumqttd::Error),
    #[error("mqtt operation failed: {0}")]
    LinkError(#[from] librumqttd::LinkError),
}
