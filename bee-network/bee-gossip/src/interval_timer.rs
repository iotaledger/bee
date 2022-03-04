// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use rand::{thread_rng, Rng as _};

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

pub(crate) type Pause = Duration;

// TODO: revisit dead code
#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct IntervalTimerBuilder {
    max_count: Option<usize>,
    timeout: Option<Duration>,
    jitter: Option<f32>,
    mode: TimerMode,
}

// TODO: revisit dead code
#[allow(dead_code)]
impl IntervalTimerBuilder {
    pub fn new(mode: TimerMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn with_max_count(mut self, max_count: usize) -> Self {
        self.max_count.replace(max_count);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout.replace(timeout);
        self
    }

    pub fn with_jitter(mut self, jitter: f32) -> Self {
        assert!((0.0..=1.0).contains(&jitter));

        self.jitter.replace(jitter);
        self
    }

    pub fn finish(self) -> IntervalTimer {
        IntervalTimer {
            max_count: self.max_count.unwrap_or(usize::MAX),
            timeout: self.timeout.unwrap_or(Duration::MAX),
            jitter: self.jitter.unwrap_or(1.0),
            mode: self.mode,
            curr_count: 0,
            timestamp: Instant::now(),
        }
    }
}

pub(crate) struct IntervalTimer {
    max_count: usize,
    timeout: Duration,
    jitter: f32,
    mode: TimerMode,
    curr_count: usize,
    timestamp: Instant,
}

impl Iterator for IntervalTimer {
    type Item = Pause;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_count >= self.max_count
            || Instant::now()
                .checked_duration_since(self.timestamp)
                .expect("error duration since")
                > self.timeout
        {
            None
        } else {
            let mut next_interval_millis = match &mut self.mode {
                TimerMode::Constant(value) => *value,
                TimerMode::Exponential(value, factor) => {
                    let prev_value = *value;
                    *value = (*value as f32 * *factor) as u64;
                    prev_value
                }
            };
            self.curr_count += 1;

            if (self.jitter - 1.0).abs() > f32::EPSILON {
                next_interval_millis =
                    thread_rng().gen_range(((next_interval_millis as f32 * self.jitter) as u64)..next_interval_millis)
            }

            Some(Pause::from_millis(next_interval_millis))
        }
    }
}

pub(crate) enum TimerMode {
    Constant(u64),
    //Linear(u64, u64),
    Exponential(u64, f32),
}

impl Default for TimerMode {
    fn default() -> Self {
        Self::Constant(0)
    }
}

pub(crate) struct ManualTimer(AtomicU64);

impl ManualTimer {
    pub const fn new(initial_delay: Duration) -> Self {
        Self(AtomicU64::new(pause_to_u64(initial_delay)))
    }

    /// Defines the delays produced by the factory.
    ///
    /// There's no corresponding `get` method. Use the `next` method ([`Iterator`] trait impl) to access them.
    pub fn set(&self, pause: Pause) {
        self.0.store(pause_to_u64(pause), Ordering::Relaxed);
    }
}

const fn pause_to_u64(delay: Pause) -> u64 {
    // Type cast: for all practical purposes this should be fine.
    delay.as_millis() as u64
}

const fn u64_to_pause(pause: u64) -> Pause {
    Pause::from_millis(pause)
}

impl Iterator for ManualTimer {
    type Item = Pause;

    fn next(&mut self) -> Option<Self::Item> {
        Some(u64_to_pause(self.0.load(Ordering::Relaxed)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_zero_pause() {
        let mut timer = IntervalTimerBuilder::new(TimerMode::Constant(0))
            .with_max_count(4)
            .finish();

        assert_eq!(0, timer.next().unwrap().as_millis());
        assert_eq!(0, timer.next().unwrap().as_millis());
        assert_eq!(0, timer.next().unwrap().as_millis());
        assert_eq!(0, timer.next().unwrap().as_millis());
        assert_eq!(None, timer.next());
        assert_eq!(None, timer.next());
    }

    #[test]
    fn constant_positive_pause() {
        let mut timer = IntervalTimerBuilder::new(TimerMode::Constant(500))
            .with_max_count(4)
            .finish();

        assert_eq!(500, timer.next().unwrap().as_millis());
        assert_eq!(500, timer.next().unwrap().as_millis());
        assert_eq!(500, timer.next().unwrap().as_millis());
        assert_eq!(500, timer.next().unwrap().as_millis());
        assert_eq!(None, timer.next());
        assert_eq!(None, timer.next());
    }

    #[test]
    fn exponential_pause() {
        let mut timer = IntervalTimerBuilder::new(TimerMode::Exponential(100, 2.0))
            .with_max_count(4)
            .finish();

        assert_eq!(100, timer.next().unwrap().as_millis());
        assert_eq!(200, timer.next().unwrap().as_millis());
        assert_eq!(400, timer.next().unwrap().as_millis());
        assert_eq!(800, timer.next().unwrap().as_millis());
        assert_eq!(None, timer.next());
        assert_eq!(None, timer.next());
    }

    #[test]
    fn constant_pause_with_jitter() {
        let mut delay = IntervalTimerBuilder::new(TimerMode::Constant(500))
            .with_max_count(4)
            .with_jitter(0.5)
            .finish();

        assert!((250..=500).contains(&(delay.next().unwrap().as_millis() as u64)));
        assert!((250..=500).contains(&(delay.next().unwrap().as_millis() as u64)));
        assert!((250..=500).contains(&(delay.next().unwrap().as_millis() as u64)));
        assert!((250..=500).contains(&(delay.next().unwrap().as_millis() as u64)));
        assert_eq!(None, delay.next());
        assert_eq!(None, delay.next());
    }

    #[tokio::test]
    async fn constant_pause_with_timeout() {
        let mut delay = IntervalTimerBuilder::new(TimerMode::Constant(25))
            .with_max_count(4)
            .with_timeout(Duration::from_millis(50))
            .finish();

        assert_eq!(25, delay.next().unwrap().as_millis());
        tokio::time::sleep(Duration::from_millis(25)).await;
        assert_eq!(25, delay.next().unwrap().as_millis());
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(None, delay.next());
        assert_eq!(None, delay.next());
    }
}
