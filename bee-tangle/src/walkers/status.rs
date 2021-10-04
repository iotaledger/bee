// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageData;

use bee_message::MessageId;

///
#[derive(Debug)]
pub enum TangleWalkerStatus {
    ///
    Matched(MessageId, MessageData),
    ///
    Skipped(MessageId, MessageData),
    ///
    Missing(MessageId),
}
