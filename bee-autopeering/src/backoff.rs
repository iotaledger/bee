// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

pub(crate) struct Backoff {
    max_retries: usize,
    current_retries: usize,
    mode: BackoffMode,
}

impl Backoff {
    pub fn new(max_retries: usize, mode: BackoffMode) -> Self {
        Self {
            max_retries,
            current_retries: 0,
            mode,
        }
    }
}
pub(crate) enum BackoffMode {
    Zero,
    Constant(u64),
    Exponential(u64, f32),
}

impl Iterator for Backoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_retries >= self.max_retries {
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

            Some(Duration::from_secs(next_interval_secs))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_backoff() {
        let mut backoff = Backoff::new(4, BackoffMode::Zero);

        const ZERO: Duration = Duration::from_secs(0);

        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(Some(ZERO), backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn constant_backoff() {
        let mut backoff = Backoff::new(4, BackoffMode::Constant(5));

        const FIVE: Duration = Duration::from_secs(5);

        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(Some(FIVE), backoff.next());
        assert_eq!(None, backoff.next());
    }

    #[test]
    fn exponential_backoff() {
        let mut backoff = Backoff::new(4, BackoffMode::Exponential(100, 2.0));

        assert_eq!(Some(Duration::from_secs(100)), backoff.next());
        assert_eq!(Some(Duration::from_secs(200)), backoff.next());
        assert_eq!(Some(Duration::from_secs(400)), backoff.next());
        assert_eq!(Some(Duration::from_secs(800)), backoff.next());
        assert_eq!(None, backoff.next());
    }
}
