// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use futures::{
    future::FusedFuture,
    stream::{FusedStream, Stream},
};

use std::{
    collections::{BinaryHeap, VecDeque},
    future::Future,
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll, Waker},
};

pub struct WaitPriorityQueue<T: Ord + Eq> {
    inner: Mutex<(BinaryHeap<T>, VecDeque<Waker>)>,
}

impl<T: Ord + Eq> WaitPriorityQueue<T> {
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().0.is_empty()
    }
}

impl<T: Ord + Eq> Default for WaitPriorityQueue<T> {
    fn default() -> Self {
        Self {
            inner: Mutex::new((BinaryHeap::new(), VecDeque::new())),
        }
    }
}

impl<T: Ord + Eq> WaitPriorityQueue<T> {
    /// Pushes an item into the queue. It will be removed in an order consistent with the ordering
    /// of itself relative to other items in the queue at the time of removal.
    pub fn push(&self, entry: T) {
        let mut inner = self.inner.lock().unwrap();

        inner.0.push(entry);

        if let Some(a) = inner.1.pop_front() {
            Waker::wake(a)
        }
    }

    /// Attempts to remove the item with the highest priority from the queue, returning [`None`] if
    /// there are no available items.
    pub fn try_pop(&self) -> Option<T> {
        self.inner.lock().unwrap().0.pop()
    }

    /// Removes the item with the highest priority from the queue, waiting on an item should there
    /// not be one immediately available.
    pub fn pop(&self) -> impl Future<Output = T> + FusedFuture + '_ {
        WaitFut {
            queue: self,
            terminated: false,
        }
    }

    /// Returns a stream of highest-priority items from this queue.
    pub fn incoming(&self) -> WaitIncoming<T> {
        WaitIncoming { queue: self }
    }

    /// Returns an iterator of pending items from this queue (i.e: those that are immediately
    /// available).
    pub fn pending(&self) -> impl Iterator<Item = T> + '_ {
        std::iter::from_fn(move || self.inner.lock().unwrap().0.pop())
    }
}

pub(crate) struct WaitFut<'a, T: Ord + Eq> {
    queue: &'a WaitPriorityQueue<T>,
    terminated: bool,
}

impl<'a, T: Ord + Eq> Future for WaitFut<'a, T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.queue.inner.lock().unwrap();

        match inner.0.pop() {
            _ if self.terminated => Poll::Pending,
            Some(entry) => {
                self.terminated = true;
                Poll::Ready(entry)
            }
            None => {
                inner.1.push_back(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<'a, T: Ord + Eq> FusedFuture for WaitFut<'a, T> {
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}

pub struct WaitIncoming<'a, T: Ord + Eq> {
    queue: &'a WaitPriorityQueue<T>,
}

impl<'a, T: Ord + Eq> Stream for WaitIncoming<'a, T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut inner = self.queue.inner.lock().unwrap();

        match inner.0.pop() {
            Some(entry) => Poll::Ready(Some(entry)),
            None => {
                inner.1.push_back(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<'a, T: Ord + Eq> FusedStream for WaitIncoming<'a, T> {
    fn is_terminated(&self) -> bool {
        false
    }
}
