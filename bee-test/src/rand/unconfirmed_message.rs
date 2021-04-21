// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::message::rand_message_id;

use bee_tangle::unconfirmed_message::UnconfirmedMessage;

pub fn rand_unconfirmed_message() -> UnconfirmedMessage {
    rand_message_id().into()
}
