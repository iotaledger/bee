// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::{output::Output, spent::Spent};
use bee_message::{
    payload::transaction::{Ed25519Address, OutputId},
    prelude::HashedIndex,
    MessageId,
};
use bee_storage::{access::Fetch, storage};

pub trait Backend:
    storage::Backend
    + Fetch<HashedIndex, Vec<MessageId>>
    + Fetch<OutputId, Output>
    + Fetch<OutputId, Spent>
    + Fetch<Ed25519Address, Vec<OutputId>>
{
}

impl<T> Backend for T where
    T: storage::Backend
        + Fetch<HashedIndex, Vec<MessageId>>
        + Fetch<OutputId, Output>
        + Fetch<OutputId, Spent>
        + Fetch<Ed25519Address, Vec<OutputId>>
{
}
