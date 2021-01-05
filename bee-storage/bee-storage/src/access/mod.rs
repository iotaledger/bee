// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Batch module which holds the contract for batch access operation for all backends
pub mod batch;
/// Delete module which holds the contract for delete access operation for all backends
pub mod delete;
/// Exist module which holds the contract for exist access operation for all backends
pub mod exist;
/// Fetch module which holds the contract for fetch access operation for all backends
pub mod fetch;
/// Insert module which holds the contract for insert access operation for all backends
pub mod insert;
/// Stream module which holds the contract for stream-like access operations for all backends
pub mod stream;

pub use batch::{Batch, BatchBuilder};
pub use delete::Delete;
pub use exist::Exist;
pub use fetch::Fetch;
pub use insert::Insert;
pub use stream::AsStream;
