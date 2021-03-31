// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! FPC statements for performing and recording queries.

mod entry;

mod conflict;
mod opinion;
mod registry;
mod timestamp;

pub use conflict::Conflict;
pub use opinion::{Opinion, Opinions, OPINION_STATEMENT_LENGTH};
pub use registry::Registry;
pub use timestamp::Timestamp;
