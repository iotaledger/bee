// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Holds the contract for batch access operation.
pub mod batch;
/// Holds the contract for delete access operation.
pub mod delete;
/// Holds the contract for exist access operation.
pub mod exist;
/// Holds the contract for fetch access operation.
pub mod fetch;
/// Holds the contract for insert access operation.
pub mod insert;
/// Holds the contract for stream access operations.
pub mod stream;
/// Holds the contract for truncate access operations.
pub mod truncate;

pub use batch::{Batch, BatchBuilder};
pub use delete::Delete;
pub use exist::Exist;
pub use fetch::Fetch;
pub use insert::Insert;
pub use stream::AsStream;
pub use truncate::Truncate;
