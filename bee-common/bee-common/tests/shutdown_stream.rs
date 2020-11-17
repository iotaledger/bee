// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::shutdown_stream::ShutdownStream;

use async_std::task;
use futures::{
    channel::{mpsc, oneshot},
    SinkExt, StreamExt,
};

use std::time::Duration;

#[async_std::test]
async fn no_shutdown() {
    let (mut sender, receiver) = mpsc::unbounded::<usize>();
    let (_shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();
    let handle = task::spawn(async move {
        let mut shutdown_stream = ShutdownStream::new(shutdown_receiver, receiver);

        let mut acc = 0;

        while let Some(item) = shutdown_stream.next().await {
            acc += item;
            task::sleep(Duration::from_millis(5)).await;
        }

        acc
    });

    for i in 0..=100 {
        assert!(sender.send(i).await.is_ok());
        task::sleep(Duration::from_millis(5)).await;
    }

    task::sleep(Duration::from_millis(5)).await;

    sender.disconnect();

    assert_eq!(handle.await, 5050);
}

#[async_std::test]
async fn early_shutdown() {
    let (mut sender, receiver) = mpsc::unbounded::<usize>();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();
    let handle = task::spawn(async move {
        let mut shutdown_stream = ShutdownStream::new(shutdown_receiver, receiver);

        let mut acc = 0;

        while let Some(item) = shutdown_stream.next().await {
            acc += item;
            task::sleep(Duration::from_millis(1)).await;
        }

        acc
    });

    for i in 0..=100 {
        assert!(sender.send(i).await.is_ok());
    }

    assert!(shutdown_sender.send(()).is_ok());

    assert!(handle.await < 5050);
}
