// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_runtime::shutdown_stream::ShutdownStream;

use futures::{
    channel::{mpsc, oneshot},
    SinkExt, StreamExt,
};
use tokio::{task::spawn, time::sleep};

use std::time::Duration;

#[tokio::test]
async fn no_shutdown() {
    let (mut sender, receiver) = mpsc::unbounded::<usize>();
    let (_shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    let handle = spawn(async move {
        let mut shutdown_stream = ShutdownStream::new(shutdown_receiver, receiver);

        let mut acc = 0;

        while let Some(item) = shutdown_stream.next().await {
            acc += item;
            sleep(Duration::from_millis(5)).await;
        }

        acc
    });

    for i in 0..=100 {
        assert!(sender.send(i).await.is_ok());
        sleep(Duration::from_millis(5)).await;
    }

    sleep(Duration::from_millis(5)).await;

    sender.disconnect();

    assert_eq!(handle.await.unwrap(), 5050);
}

#[tokio::test]
async fn early_shutdown() {
    let (sender, receiver) = mpsc::unbounded::<usize>();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    let handle = spawn(async move {
        let mut shutdown_stream = ShutdownStream::new(shutdown_receiver, receiver);

        let mut acc = 0;

        while let Some(item) = shutdown_stream.next().await {
            acc += item;
            sleep(Duration::from_millis(1)).await;
        }

        acc
    });

    for i in 0..=100 {
        assert!(sender.unbounded_send(i).is_ok());
    }

    assert!(shutdown_sender.send(()).is_ok());

    assert!(handle.await.unwrap() < 5050);
}

#[tokio::test]
async fn early_shutdown_split_from_fused() {
    let (sender, receiver) = mpsc::unbounded::<usize>();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    let handle = spawn(async move {
        let shutdown_stream = ShutdownStream::new(shutdown_receiver, receiver);
        let (shutdown, stream) = shutdown_stream.split();
        let mut shutdown_stream = ShutdownStream::from_fused(shutdown, stream);

        let mut acc = 0;

        while let Some(item) = shutdown_stream.next().await {
            acc += item;
            sleep(Duration::from_millis(1)).await;
        }

        acc
    });

    for i in 0..=100 {
        assert!(sender.unbounded_send(i).is_ok());
    }

    assert!(shutdown_sender.send(()).is_ok());

    assert!(handle.await.unwrap() < 5050);
}
