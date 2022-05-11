// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A multi-producer, single-consumer queue for sending values between asynchronous tasks.
//!
//! For more information about the specific semantics of the channel see [`tokio::sync::mpsc`].

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::stream::{Stream, StreamExt};
use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder},
    metrics::{counter::Counter, MetricType},
};
use tokio::sync::mpsc::{
    error::{SendError, TryRecvError},
    unbounded_channel as tokio_unbounded_channel, UnboundedReceiver as TokioUnboundedReceiver,
    UnboundedSender as TokioUnboundedSender,
};
use tokio_stream::wrappers::UnboundedReceiverStream as TokioUnboundedReceiverStream;

/// Counter tracking the number of received messages through an [`UnboundedReceiver`].
pub struct UnboundedReceiverCounter(Counter);

impl EncodeMetric for UnboundedReceiverCounter {
    #[inline]
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.0.encode(encoder)
    }

    #[inline]
    fn metric_type(&self) -> MetricType {
        self.0.metric_type()
    }
}

/// A wrapper around [`UnboundedReceiver`] that implements [`Stream`].
pub struct UnboundedReceiverStream<T>(TokioUnboundedReceiverStream<T>, Counter);

impl<T> UnboundedReceiverStream<T> {
    /// Create a new [`UnboundedReceiverStream`].
    #[inline]
    pub fn new(recv: UnboundedReceiver<T>) -> Self {
        Self(TokioUnboundedReceiverStream::new(recv.0), recv.1)
    }

    /// Get back the inner [`UnboundedReceiver`].
    #[inline]
    pub fn into_inner(self) -> UnboundedReceiver<T> {
        UnboundedReceiver(self.0.into_inner(), self.1)
    }

    /// Closes the receiving half of a channel without dropping it.
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered.
    #[inline]
    pub fn close(&mut self) {
        self.0.close()
    }
}

impl<T> Stream for UnboundedReceiverStream<T> {
    type Item = <TokioUnboundedReceiverStream<T> as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx).map(|opt| {
            opt.map(|message| {
                self.1.inc();
                message
            })
        })
    }
}

/// Receive values from the associated [`UnboundedSender`].
pub struct UnboundedReceiver<T>(TokioUnboundedReceiver<T>, Counter);

impl<T> UnboundedReceiver<T> {
    /// Receives the next value for this receiver and increases the counter by one if the value is
    /// not `None`.
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await.map(|message| {
            self.1.inc();
            message
        })
    }

    /// Tries to receive the next value for this receiver and increases the counter by one if
    /// successful.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.0.try_recv().map(|message| {
            self.1.inc();
            message
        })
    }

    /// Blocking receive to call outside of asynchronous contexts.
    pub fn blocking_recv(&mut self) -> Option<T> {
        self.0.blocking_recv().map(|message| {
            self.1.inc();
            message
        })
    }

    /// Closes the receiving half of a channel, without dropping it.
    #[inline]
    pub fn close(&mut self) {
        self.0.close()
    }

    /// Polls to receive the next message on this channel.
    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.0.poll_recv(cx).map(|opt| {
            opt.map(|message| {
                self.1.inc();
                message
            })
        })
    }
}

/// Counter tracking the number of received messages through an [`UnboundedSender`].
pub struct UnboundedSenderCounter(Counter);

impl EncodeMetric for UnboundedSenderCounter {
    #[inline]
    fn encode(&self, encoder: Encoder) -> Result<(), std::io::Error> {
        self.0.encode(encoder)
    }

    #[inline]
    fn metric_type(&self) -> MetricType {
        self.0.metric_type()
    }
}

/// Send values to the associated [`UnboundedReceiver`].
pub struct UnboundedSender<T>(TokioUnboundedSender<T>, Counter);

impl<T> UnboundedSender<T> {
    /// Attempts to send a message on this [`UnboundedSender`] without blocking and increases the
    /// counter by one if successful.
    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.0.send(message).map(|()| {
            self.1.inc();
        })
    }

    /// Completes when the receiver has dropped.
    #[inline]
    pub async fn closed(&self) {
        self.0.closed().await
    }

    /// Checks if the channel has been closed.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    /// Returns `true` if senders belong to the same channel.
    #[inline]
    pub fn same_channel(&self, other: &Self) -> bool {
        self.0.same_channel(&other.0)
    }
}

impl<T> Clone for UnboundedSender<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

/// Creates an unbounded mpsc channel for communicating between asynchronous tasks without backpressure.
pub fn unbounded_channel<T>() -> (
    UnboundedSender<T>,
    UnboundedReceiver<T>,
    UnboundedSenderCounter,
    UnboundedReceiverCounter,
) {
    let tx_counter = Counter::default();
    let rx_counter = Counter::default();

    let (tx, rx) = tokio_unbounded_channel();

    (
        UnboundedSender(tx, tx_counter.clone()),
        UnboundedReceiver(rx, rx_counter.clone()),
        UnboundedSenderCounter(tx_counter),
        UnboundedReceiverCounter(rx_counter),
    )
}
