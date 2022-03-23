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
/// Holds the contract for iter access operations.
mod iter;
/// Holds the contract for multiple fetch access operation.
mod multi_fetch;
/// Holds the contract for truncate access operations.
mod truncate;
/// Holds the contract for update access operations.
mod update;

pub use self::{
    batch::{Batch, BatchBuilder},
    delete::Delete,
    exist::Exist,
    fetch::Fetch,
    insert::{Insert, InsertStrict},
    iter::AsIterator,
    multi_fetch::MultiFetch,
    truncate::Truncate,
    update::Update,
};
