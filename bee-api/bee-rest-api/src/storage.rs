// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::model::{Output, Spent};
use bee_message::{
    payload::transaction::{Ed25519Address, OutputId},
    prelude::HashedIndex,
    MessageId,
};
use bee_storage::{
    access::{Exist, Fetch},
    storage,
};

pub trait Backend:
    storage::Backend
    + Exist<OutputId, Spent>
    + Fetch<HashedIndex, Vec<MessageId>>
    + Fetch<OutputId, Output>
    + Fetch<OutputId, Spent>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + bee_protocol::storage::Backend
{
}

impl<T> Backend for T where
    T: storage::Backend
        + Exist<OutputId, Spent>
        + Fetch<HashedIndex, Vec<MessageId>>
        + Fetch<OutputId, Output>
        + Fetch<OutputId, Spent>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + bee_protocol::storage::Backend
{
}
