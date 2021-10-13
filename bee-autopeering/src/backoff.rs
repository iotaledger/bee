// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::time;

use rand::{thread_rng, Rng as _};

use std::time::Duration;

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
            timestamp: time::unix_now(),
        }
    }
}

pub(crate) struct Backoff {
    max_retries: usize,
    timeout: Duration,
    jitter: f32,
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

        const MILLIS_0: Duration = Duration::from_millis(0);

        assert_eq!(Some(MILLIS_0), backoff.next());
        assert_eq!(Some(MILLIS_0), backoff.next());
        assert_eq!(Some(MILLIS_0), backoff.next());
        assert_eq!(Some(MILLIS_0), backoff.next());
        assert_eq!(None, backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn constant_backoff() {
        let mut backoff = BackoffBuilder::new(BackoffMode::Constant(5))
            .with_max_retries(4)
            .finish();

        const MILLIS_500: Duration = Duration::from_millis(500);

        assert_eq!(Some(MILLIS_500), backoff.next());
        assert_eq!(Some(MILLIS_500), backoff.next());
        assert_eq!(Some(MILLIS_500), backoff.next());
        assert_eq!(Some(MILLIS_500), backoff.next());
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
