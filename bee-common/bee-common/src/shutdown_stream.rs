// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! A module to simplify selecting between a shutdown signal and a stream.
//!
//! The `ShutdownStream` type can be used to replace this pattern:
//! ```ignore
//! loop {
//!     select! {
//!         _ = shutdown => break,
//!         item = stream.next() => { /* actual logic */ },
//!     }
//! }
//! ```
//! by this one:
//! ```ignore
//! let mut shutdown_stream = ShutdownStream::new(stream, shutdown);
//!
//! while let Some(item) = shutdown_stream.next().await {
//!     /* actual logic */
//! }
//! ```
use futures::{
    channel::oneshot::Receiver,
    future, stream,
    task::{Context, Poll},
    FutureExt, Stream, StreamExt,
};

use std::pin::Pin;
/// A stream with a shutdown.
///
/// This type wraps a regular stream and a shutdown receiver to produce a new stream that ends when
/// the shutdown receiver is triggered or when the stream ends.
pub struct ShutdownStream<S> {
    stream: stream::Fuse<S>,
    shutdown: future::Fuse<Receiver<()>>,
}

impl<S: Stream> ShutdownStream<S> {
    /// Create a new `ShutdownStream`.
    ///
    /// This method receives the stream to be wrapped and a `oneshot::Receiver` for the shutdown.
    /// Both the stream and the shutdown receiver are fused to avoid polling already completed
    /// futures.
    pub fn new(stream: S, shutdown: Receiver<()>) -> Self {
        Self {
            stream: stream.fuse(),
            shutdown: shutdown.fuse(),
        }
    }
}

impl<S: Stream<Item = T> + std::marker::Unpin, T> Stream for ShutdownStream<S> {
    type Item = T;
    /// The shutdown receiver is polled first, if it is not ready, the stream is polled. This
    /// guarantees that checking for shutdown always happens first.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(_) = self.shutdown.poll_unpin(cx) {
            Poll::Ready(None)
        } else {
            self.stream.poll_next_unpin(cx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let mut shutdown_stream = ShutdownStream::new(receiver, shutdown_receiver);

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
            let mut shutdown_stream = ShutdownStream::new(receiver, shutdown_receiver);

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
}
