// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::number::rand_number;

use bee_message::payload::drng::ApplicationMessagePayload;

/// Generates a random [`ApplicationMessagePayload`].
pub fn rand_application_message_payload() -> ApplicationMessagePayload {
    ApplicationMessagePayload::new(rand_number())
}
