// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageData;

use bee_message::MessageId;

/// The type yielded by [`Tangle`](crate::Tangle) walkers.
#[derive(Debug)]
pub enum TangleWalkerItem {
    /// The item matched the walk condition.
    Matched(MessageId, MessageData),
    /// The item did not match the walk condition.
    Skipped(MessageId, MessageData),
    /// The item is missing from the tangle.
    Missing(MessageId),
}
