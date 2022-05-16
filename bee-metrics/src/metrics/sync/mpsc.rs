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
pub struct UnboundedReceiverStream<T> {
    stream: TokioUnboundedReceiverStream<T>,
    counter: Counter,
}

impl<T> UnboundedReceiverStream<T> {
    /// Creates a new [`UnboundedReceiverStream`].
    #[inline]
    pub fn new(recv: UnboundedReceiver<T>) -> Self {
        Self {
            stream: TokioUnboundedReceiverStream::new(recv.receiver),
            counter: recv.counter,
        }
    }

    /// Get back the inner [`UnboundedReceiver`].
    #[inline]
    pub fn into_inner(self) -> UnboundedReceiver<T> {
        UnboundedReceiver {
            receiver: self.stream.into_inner(),
            counter: self.counter,
        }
    }

    /// Closes the receiving half of a channel without dropping it.
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered.
    #[inline]
    pub fn close(&mut self) {
        self.stream.close()
    }
}

impl<T> Stream for UnboundedReceiverStream<T> {
    type Item = <TokioUnboundedReceiverStream<T> as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx).map(|opt| {
            opt.map(|message| {
                self.counter.inc();
                message
            })
        })
    }
}

/// Receive values from the associated [`UnboundedSender`].
pub struct UnboundedReceiver<T> {
    receiver: TokioUnboundedReceiver<T>,
    counter: Counter,
}

impl<T> UnboundedReceiver<T> {
    /// Returns the counter metric for this side of the channel.
    pub fn counter(&self) -> UnboundedReceiverCounter {
        UnboundedReceiverCounter(self.counter.clone())
    }

    /// Receives the next value for this receiver and increases the counter by one if the value is
    /// not `None`.
    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await.map(|message| {
            self.counter.inc();
            message
        })
    }

    /// Tries to receive the next value for this receiver and increases the counter by one if
    /// successful.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.receiver.try_recv().map(|message| {
            self.counter.inc();
            message
        })
    }

    /// Blocking receive to call outside of asynchronous contexts.
    pub fn blocking_recv(&mut self) -> Option<T> {
        self.receiver.blocking_recv().map(|message| {
            self.counter.inc();
            message
        })
    }

    /// Closes the receiving half of a channel, without dropping it.
    #[inline]
    pub fn close(&mut self) {
        self.receiver.close()
    }

    /// Polls to receive the next message on this channel.
    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.receiver.poll_recv(cx).map(|opt| {
            opt.map(|message| {
                self.counter.inc();
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
pub struct UnboundedSender<T> {
    sender: TokioUnboundedSender<T>,
    counter: Counter,
}

impl<T> UnboundedSender<T> {
    /// Returns the counter metric for this side of the channel.
    pub fn counter(&self) -> UnboundedSenderCounter {
        UnboundedSenderCounter(self.counter.clone())
    }

    /// Attempts to send a message on this [`UnboundedSender`] without blocking and increases the
    /// counter by one if successful.
    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        self.sender.send(message).map(|()| {
            self.counter.inc();
        })
    }

    /// Completes when the receiver has dropped.
    #[inline]
    pub async fn closed(&self) {
        self.sender.closed().await
    }

    /// Checks if the channel has been closed.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    /// Returns `true` if senders belong to the same channel.
    #[inline]
    pub fn same_channel(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender)
    }
}

impl<T> Clone for UnboundedSender<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            counter: self.counter.clone(),
        }
    }
}

/// Creates an unbounded mpsc channel for communicating between asynchronous tasks without backpressure.
pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (sender, receiver) = tokio_unbounded_channel();

    (
        UnboundedSender {
            sender,
            counter: Counter::default(),
        },
        UnboundedReceiver {
            receiver,
            counter: Counter::default(),
        },
    )
}
