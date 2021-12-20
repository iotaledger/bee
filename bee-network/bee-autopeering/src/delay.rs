// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

pub(crate) type Delay = Duration;

pub(crate) struct ManualDelayFactory(AtomicU64);

impl ManualDelayFactory {
    /// Creates a new `ManualDelayFactory` from an initial delay.
    pub(crate) const fn new(initial_delay: Delay) -> Self {
        Self(AtomicU64::new(delay_to_millis(initial_delay)))
    }

    /// Defines the delays produced by the factory.
    ///
    /// There's no corresponding `get` method. Use the `next` method ([`Iterator`] trait impl) to access them.
    pub(crate) fn set(&self, delay: Delay) {
        self.0.store(delay_to_millis(delay), Ordering::Relaxed);
    }
}

const fn delay_to_millis(delay: Delay) -> u64 {
    // Type cast: for all practical purposes this should be fine.
    delay.as_millis() as u64
}

const fn millis_to_delay(millis: u64) -> Delay {
    Delay::from_millis(millis)
}

impl Iterator for ManualDelayFactory {
    type Item = Delay;

    fn next(&mut self) -> Option<Self::Item> {
        Some(millis_to_delay(self.0.load(Ordering::Relaxed)))
    }
}
