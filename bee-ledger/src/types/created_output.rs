// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::Error;

use bee_message::{output::Output, MessageId};
use bee_packable::Packable;

use core::ops::Deref;

/// Represents a newly created output.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[packable(unpack_error = Error)]
pub struct CreatedOutput {
    message_id: MessageId,
    inner: Output,
}

impl CreatedOutput {
    /// Creates a new `CreatedOutput`.
    pub fn new(message_id: MessageId, inner: Output) -> Self {
        Self { message_id, inner }
    }

    /// Returns the message id of the `CreatedOutput`.
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the inner output of the `CreatedOutput`.
    pub fn inner(&self) -> &Output {
        &self.inner
    }
}

impl Deref for CreatedOutput {
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
