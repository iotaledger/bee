// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::Tangle;
use bee_test::rand::message::{metadata::rand_message_metadata, rand_message};

#[tokio::test]
async fn insert_get() {
    let (message, metadata) = (rand_message(), rand_message_metadata());
    let message_id = message.id();

    let tangle = Tangle::new();
    tangle.insert(message_id, message.clone(), metadata.clone()).await;

    let message_data = tangle.get(&message_id).await.unwrap();

    assert_eq!(*message_data.message(), message);
    assert_eq!(*message_data.metadata(), metadata);
}
