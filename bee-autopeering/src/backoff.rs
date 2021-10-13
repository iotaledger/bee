// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::time;

use rand::{thread_rng, Rng as _};

use std::time::Duration;

#[derive(Default)]
pub(crate) struct BackoffBuilder {
    max_retries: Option<usize>,
    timeout: Option<Duration>,
    jitter: Option<i64>,
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

    pub fn with_jitter(mut self, jitter: i64) -> Self {
        self.jitter.replace(jitter);
        self
    }

    pub fn finish(self) -> Backoff {
        Backoff {
            max_retries: self.max_retries.unwrap_or(usize::MAX),
            timeout: self.timeout.unwrap_or(Duration::MAX),
            jitter: self.jitter.unwrap_or(0),
            mode: self.mode,
            current_retries: 0,
            timestamp: time::unix_now(),
        }
    }
}

pub(crate) struct Backoff {
    max_retries: usize,
    timeout: Duration,
    jitter: i64,
    mode: BackoffMode,
    current_retries: usize,
    timestamp: u64,
}

impl Iterator for Backoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_retries >= self.max_retries {
            None
        } else if Duration::from_secs(time::unix_now() - self.timestamp) > self.timeout {
            None
        } else {
            let next_interval_secs = match &mut self.mode {
                BackoffMode::Zero => 0,
                BackoffMode::Constant(value) => *value,
                BackoffMode::Exponential(value, factor) => {
                    let prev_value = *value;
                    *value = (*value as f32 * *factor) as u64;
                    prev_value
                }
            };
            self.current_retries += 1;

            let jitter = if self.jitter != 0 {
                thread_rng().gen_range(-self.jitter..self.jitter)
            } else {
                0
            };

            Some(Duration::from_secs((next_interval_secs as i64 - jitter) as u64))
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

        const ZERO: Duration = Duration::from_secs(0);

        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn constant_backoff() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Constant(5))
            .with_max_retries(4)
            .finish();

        const FIVE: Duration = Duration::from_secs(5);

        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn exponential_backoff() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Exponential(100, 2.0))
            .with_max_retries(4)
            .finish();

        assert_eq!(Some(Duration::from_secs(100)), backoff.next());
        assert_eq!(Some(Duration::from_secs(200)), backoff.next());
        assert_eq!(Some(Duration::from_secs(400)), backoff.next());
        assert_eq!(Some(Duration::from_secs(800)), backoff.next());
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }
}
