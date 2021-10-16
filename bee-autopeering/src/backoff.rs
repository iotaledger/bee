// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::time;

use rand::{thread_rng, Rng as _};

use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct BackoffBuilder {
    max_retries: Option<usize>,
    timeout: Option<Duration>,
    jitter: Option<f32>,
    mode: BackoffMode,
}

impl BackoffBuilder {
    pub fn new(mode: BackoffMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries.replace(max_retries);
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

    pub fn finish(self) -> Backoff {
        Backoff {
            max_retries: self.max_retries.unwrap_or(usize::MAX),
            timeout: self.timeout.unwrap_or(Duration::MAX),
            jitter: self.jitter.unwrap_or(1.0),
            mode: self.mode,
            current_retries: 0,
            timestamp: Instant::now(),
        }
    }
}

pub(crate) struct Backoff {
    max_retries: usize,
    timeout: Duration,
    jitter: f32,
    mode: BackoffMode,
    current_retries: usize,
    timestamp: Instant,
}

impl Iterator for Backoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_retries >= self.max_retries {
            None
        } else if Instant::now()
            .checked_duration_since(self.timestamp)
            .expect("error duration since")
            > self.timeout
        {
            None
        } else {
            let mut next_interval_millis = match &mut self.mode {
                BackoffMode::Zero => 0,
                BackoffMode::Constant(value) => *value,
                BackoffMode::Exponential(value, factor) => {
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

pub(crate) enum BackoffMode {
    Zero,
    Constant(u64),
    Exponential(u64, f32),
}

impl Default for BackoffMode {
    fn default() -> Self {
        Self::Zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_backoff() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Zero).with_max_retries(4).finish();

        assert_eq!(0, backoff.next().unwrap().as_millis());
        assert_eq!(0, backoff.next().unwrap().as_millis());
        assert_eq!(0, backoff.next().unwrap().as_millis());
        assert_eq!(0, backoff.next().unwrap().as_millis());
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn constant_backoff() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Constant(500))
            .with_max_retries(4)
            .finish();

        assert_eq!(500, backoff.next().unwrap().as_millis());
        assert_eq!(500, backoff.next().unwrap().as_millis());
        assert_eq!(500, backoff.next().unwrap().as_millis());
        assert_eq!(500, backoff.next().unwrap().as_millis());
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn exponential_backoff() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Exponential(100, 2.0))
            .with_max_retries(4)
            .finish();

        assert_eq!(100, backoff.next().unwrap().as_millis());
        assert_eq!(200, backoff.next().unwrap().as_millis());
        assert_eq!(400, backoff.next().unwrap().as_millis());
        assert_eq!(800, backoff.next().unwrap().as_millis());
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn constant_backoff_with_jitter() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Constant(500))
            .with_max_retries(4)
            .with_jitter(0.5)
            .finish();

        assert!((250..=500).contains(&(backoff.next().unwrap().as_millis() as u64)));
        assert!((250..=500).contains(&(backoff.next().unwrap().as_millis() as u64)));
        assert!((250..=500).contains(&(backoff.next().unwrap().as_millis() as u64)));
        assert!((250..=500).contains(&(backoff.next().unwrap().as_millis() as u64)));
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[tokio::test]
    async fn constant_backoff_with_timeout() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Constant(25))
            .with_max_retries(4)
            .with_timeout(Duration::from_millis(50))
            .finish();

        assert_eq!(25, backoff.next().unwrap().as_millis());
        tokio::time::sleep(Duration::from_millis(25)).await;
        assert_eq!(25, backoff.next().unwrap().as_millis());
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }
}
