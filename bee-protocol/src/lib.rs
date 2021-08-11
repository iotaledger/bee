// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod parser;
mod storage;

use bee_plugin::UniqueId;
pub use parser::*;
pub use storage::*;

use std::sync::Arc;

type EventBus = Arc<bee_event_bus::EventBus<'static, UniqueId>>;
