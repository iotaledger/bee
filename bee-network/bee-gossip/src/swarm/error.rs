// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO Error.
    #[error("{0}")]
    IoError(#[from] std::io::Error),

    /// Creating noise authentication keys failed.
    #[error("creating noise authentication keys failed")]
    CreatingNoiseKeysFailed,
}
