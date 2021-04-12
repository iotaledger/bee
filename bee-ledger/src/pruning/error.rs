// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", .0)]
    MilestoneNotFoundInTangle(u32),
    #[error("{:?}", .0)]
    StorageError(Box<dyn std::error::Error + Send>),
}
