// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{node::BeeNodeBuilder, storage::StorageBackend};

use bee_common::logger::logger_init;

impl<B: StorageBackend> BeeNodeBuilder<B> {
    pub fn with_logging(self) -> Self {
        logger_init(self.config().logger.clone()).unwrap();
        self
    }
}
