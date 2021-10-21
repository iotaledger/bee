// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{shutdown::ShutdownRx, time};

use rand::{thread_rng, Rng as _};

use std::time::{Duration, Instant};

// Command run on notice
#[async_trait::async_trait]
pub(crate) trait Repeat
where
    Self: Send,
{
    type Command: Send;
    type Context: Send;

    async fn repeat(delay: Delay, cmd: Self::Command, ctx: Self::Context, shutdown_rx: ShutdownRx);
}

#[derive(Default)]
pub(crate) struct DelayBuilder {
    max_count: Option<usize>,
    timeout: Option<Duration>,
    jitter: Option<f32>,
    mode: DelayMode,
}

impl DelayBuilder {
    pub fn new(mode: DelayMode) -> Self {
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

    pub fn finish(self) -> Delay {
        Delay {
            max_count: self.max_count.unwrap_or(usize::MAX),
            timeout: self.timeout.unwrap_or(Duration::MAX),
            jitter: self.jitter.unwrap_or(1.0),
            mode: self.mode,
            current_retries: 0,
            timestamp: Instant::now(),
        }
    }
}

pub(crate) struct Delay {
    max_count: usize,
    timeout: Duration,
    jitter: f32,
    mode: DelayMode,
    current_retries: usize,
    timestamp: Instant,
}

impl Iterator for Delay {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_retries >= self.max_count {
            None
        } else if Instant::now()
            .checked_duration_since(self.timestamp)
            .expect("error duration since")
            > self.timeout
        {
            None
        } else {
            let mut next_interval_millis = match &mut self.mode {
                DelayMode::Zero => 0,
                DelayMode::Constant(value) => *value,
                DelayMode::Exponential(value, factor) => {
                    let prev_value = *value;
                    *value = (*value as f32 * *factor) as u64;
                    prev_value
                }
            };
            self.current_retries += 1;

            if self.jitter != 1.0 {
                next_interval_millis =
                    thread_rng().gen_range(((next_interval_millis as f32 * self.jitter) as u64)..next_interval_millis)
            }

            Some(Duration::from_millis(next_interval_millis))
        }
    }
}

pub(crate) enum DelayMode {
    Zero,
    Constant(u64),
    Exponential(u64, f32),
}

impl Default for DelayMode {
    fn default() -> Self {
        Self::Zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_delay() {
        let mut delay = DelayBuilder::new(DelayMode::Zero).with_max_count(4).finish();

        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(None, delay.next());
        assert_eq!(None, delay.next());
    }

    #[test]
    fn constant_delay() {
        let mut delay = DelayBuilder::new(DelayMode::Constant(500)).with_max_count(4).finish();

        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(None, delay.next());
        assert_eq!(None, delay.next());
    }

    #[test]
    fn exponential_delay() {
        let mut delay = DelayBuilder::new(DelayMode::Exponential(100, 2.0))
            .with_max_count(4)
            .finish();

        assert_eq!(100, delay.next().unwrap().as_millis());
        assert_eq!(200, delay.next().unwrap().as_millis());
        assert_eq!(400, delay.next().unwrap().as_millis());
        assert_eq!(800, delay.next().unwrap().as_millis());
        assert_eq!(None, delay.next());
        assert_eq!(None, delay.next());
    }

    #[test]
    fn constant_delay_with_jitter() {
        let mut delay = DelayBuilder::new(DelayMode::Constant(500))
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
    async fn constant_delay_with_timeout() {
        let mut delay = DelayBuilder::new(DelayMode::Constant(25))
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
