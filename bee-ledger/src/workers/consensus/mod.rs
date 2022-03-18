// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the worker required to compute and maintain the ledger state.

pub(crate) mod merkle_hasher;
pub(crate) mod metadata;
pub(crate) mod state;
pub(crate) mod white_flag;
pub(crate) mod worker;

pub use self::white_flag::white_flag;
pub use worker::{ConsensusWorker, ConsensusWorkerCommand};

pub use self::metadata::WhiteFlagMetadata;
