// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;

use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct SolidEntryPoints(Box<[MessageId]>);

impl SolidEntryPoints {
    pub fn new(seps: Box<[MessageId]>) -> Self {
        Self(seps)
    }
}

impl Deref for SolidEntryPoints {
    type Target = Box<[MessageId]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
