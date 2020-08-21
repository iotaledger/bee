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
//! let mut shutdown_stream = ShutdownStream::new(shutdown, stream);
//!
//! while let Some(item) = shutdown_stream.next().await {
//!     /* actual logic */
//! }
//! ```

use futures::{
    channel::oneshot,
    future::{self, FusedFuture},
    stream::{self, FusedStream},
    task::{Context, Poll},
    FutureExt, Stream, StreamExt,
};

use std::{marker::Unpin, pin::Pin};

/// A stream with a shutdown.
///
/// This type wraps a shutdown receiver and a stream to produce a new stream that ends when the
/// shutdown receiver is triggered or when the stream ends.
pub struct ShutdownStream<S> {
    shutdown: future::Fuse<oneshot::Receiver<()>>,
    stream: S,
}

impl<S: Stream> ShutdownStream<stream::Fuse<S>> {
    /// Create a new `ShutdownStream` from a shutdown receiver and an unfused stream.
    ///
    /// This method receives the stream to be wrapped and a `oneshot::Receiver` for the shutdown.
    /// Both the stream and the shutdown receiver are fused to avoid polling already completed
    /// futures.
    pub fn new(shutdown: oneshot::Receiver<()>, stream: S) -> Self {
        Self {
            shutdown: shutdown.fuse(),
            stream: stream.fuse(),
        }
    }
}

impl<S: Stream + FusedStream> ShutdownStream<S> {
    /// Create a new `ShutdownStream` from a shutdown receiver and a fused stream.
    ///
    /// This method receives the fused stream to be wrapped and a `oneshot::Receiver` for the shutdown.
    /// The shutdown receiver is fused to avoid polling already completed futures.
    pub fn from_fused(shutdown: oneshot::Receiver<()>, stream: S) -> Self {
        Self {
            shutdown: shutdown.fuse(),
            stream,
        }
    }
}

impl<S: Stream<Item = T> + FusedStream + Unpin, T> Stream for ShutdownStream<S> {
    type Item = T;
    /// The shutdown receiver is polled first, if it is not ready, the stream is polled. This
    /// guarantees that checking for shutdown always happens first.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if !self.shutdown.is_terminated() {
            if let Poll::Ready(_) = self.shutdown.poll_unpin(cx) {
                return Poll::Ready(None);
            }

            if !self.stream.is_terminated() {
                return self.stream.poll_next_unpin(cx);
            }
        }

        Poll::Ready(None)
    }
}

impl<S: Stream<Item = T> + FusedStream + Unpin, T> FusedStream for ShutdownStream<S> {
    fn is_terminated(&self) -> bool {
        self.shutdown.is_terminated() && self.stream.is_terminated()
    }
}
