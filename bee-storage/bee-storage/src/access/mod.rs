// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This module forms the access layer of the backend which holds the contracts of unified database access operations
//! across all the backends and Bee types.

/// Holds the contract for batch access operation.
mod batch;
/// Holds the contract for delete access operation.
mod delete;
/// Holds the contract for exist access operation.
mod exist;
/// Holds the contract for fetch access operation.
mod fetch;
/// Holds the contract for insert access operation.
mod insert;
/// Holds the contract for multiple fetch access operation.
mod multi_fetch;
/// Holds the contract for stream access operations.
mod stream;
/// Holds the contract for truncate access operations.
mod truncate;

pub use batch::{Batch, BatchBuilder};
pub use delete::Delete;
pub use exist::Exist;
pub use fetch::Fetch;
pub use insert::Insert;
pub use multi_fetch::MultiFetch;
pub use stream::AsStream;
pub use truncate::Truncate;
