// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Contains [`tracing::Subscriber`] implementation for `bee` node diagnostics.
pub mod subscriber;
/// Contains diagnostic utilities that are separate from [`tracing`].
pub mod util;

mod error;
mod observe;

pub use error::Error;
pub use observe::Observe;

pub use bee_trace_attributes::observe;
