// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod parser;

pub use parser::*;

use std::sync::Arc;

/// FIXME: use the identifier from `bee-plugin` once it is merged
type EventBus = Arc<bee_event_bus::EventBus<'static>>;
