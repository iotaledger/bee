// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::Tangle;
use bee_test::rand::message::{metadata::rand_message_metadata, rand_message};

#[tokio::test]
async fn test_insert() {
    let message = rand_message();
    let message_id = message.id();
    let metadata = rand_message_metadata();

    let tangle = Tangle::new();
    tangle.insert(message_id, message.clone(), metadata.clone()).await;

    assert_eq!(*tangle.get(&message_id).await.unwrap(), message);
    assert_eq!(tangle.get_metadata(&message_id).await.unwrap(), metadata);
}
