// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module providing different ways to walk the tangle.

mod bfs;
mod dfs;
mod item;

pub use bfs::{TangleBfsWalker, TangleBfsWalkerBuilder};
pub use dfs::{TangleDfsWalker, TangleDfsWalkerBuilder};
pub use item::TangleWalkerItem;
