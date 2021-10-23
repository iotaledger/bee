// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{shutdown::ShutdownRx, time};

use rand::{thread_rng, Rng as _};

use std::{
    future::Future,
    time::{Duration, Instant},
};

pub(crate) type Command<T: DelayedRepeat<N>, const N: usize> = Box<dyn for<'a> Fn(&'a T, &'a T::Context) + Send>;

/// A trait that allows to implement something similar to cronjobs for single types.
///
/// NOTE: The const generic is used to implement it several times for a type, if it requires several cronjobs.
#[async_trait::async_trait]
pub(crate) trait DelayedRepeat<const I: usize>
where
    Self: Send + Sync + Clone,
{
    type Context: Send;
    type Cancel: Future + Send + Unpin + 'static;

    async fn repeat(
        self,
        mut delay: DelayFactory,
        cmd: Command<Self, I>,
        ctx: Self::Context,
        mut cancel: Self::Cancel,
    ) {
        while let Some(duration) = delay.next() {
            tokio::select! {
                _ = &mut cancel => break,
                _ = time::sleep(duration) => {
                    cmd(&self, &ctx);
                }
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct DelayFactoryBuilder {
    max_count: Option<usize>,
    timeout: Option<Duration>,
    jitter: Option<f32>,
    mode: DelayFactoryMode,
}

impl DelayFactoryBuilder {
    pub fn new(mode: DelayFactoryMode) -> Self {
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

    pub fn finish(self) -> DelayFactory {
        DelayFactory {
            max_count: self.max_count.unwrap_or(usize::MAX),
            timeout: self.timeout.unwrap_or(Duration::MAX),
            jitter: self.jitter.unwrap_or(1.0),
            mode: self.mode,
            curr_count: 0,
            timestamp: Instant::now(),
        }
    }
}

/// A type that produces a series of delays (i.e. [`Duration`]s) to:
///
/// (a) implement a request backoff policy (which specifies the cooldown time between requests to a single peer),
///
/// (b) implement the [`Repeat`] trait for types like [`Local`] and [`RequestManager`], that need to run
///     maintenance in certain intervals.
pub(crate) struct DelayFactory {
    max_count: usize,
    timeout: Duration,
    jitter: f32,
    mode: DelayFactoryMode,
    curr_count: usize,
    timestamp: Instant,
}

impl Iterator for DelayFactory {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_count >= self.max_count {
            None
        } else if Instant::now()
            .checked_duration_since(self.timestamp)
            .expect("error duration since")
            > self.timeout
        {
            None
        } else {
            let mut next_interval_millis = match &mut self.mode {
                DelayFactoryMode::Zero => 0,
                DelayFactoryMode::Constant(value) => *value,
                DelayFactoryMode::Exponential(value, factor) => {
                    let prev_value = *value;
                    *value = (*value as f32 * *factor) as u64;
                    prev_value
                }
            };
            self.curr_count += 1;

            if self.jitter != 1.0 {
                next_interval_millis =
                    thread_rng().gen_range(((next_interval_millis as f32 * self.jitter) as u64)..next_interval_millis)
            }

            Some(Duration::from_millis(next_interval_millis))
        }
    }
}

/// The differnet "modus operandi" for the [`DelayFactory`].
pub(crate) enum DelayFactoryMode {
    /// The factory produces a series of 0-delays.
    Zero,
    /// The factory produces a series of constant delays. For `Constant(0)` the behavior is identical to the `Zero`
    /// mode.
    Constant(u64),
    /// The factory produces a series of exponentially growing delays.
    Exponential(u64, f32),
}

impl Default for DelayFactoryMode {
    fn default() -> Self {
        Self::Zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_delay() {
        let mut delay = DelayFactoryBuilder::new(DelayFactoryMode::Zero)
            .with_max_count(4)
            .finish();

        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(0, delay.next().unwrap().as_millis());
        assert_eq!(None, delay.next());
        assert_eq!(None, delay.next());
    }

    #[test]
    fn constant_delay() {
        let mut delay = DelayFactoryBuilder::new(DelayFactoryMode::Constant(500))
            .with_max_count(4)
            .finish();

        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(500, delay.next().unwrap().as_millis());
        assert_eq!(None, delay.next());
        assert_eq!(None, delay.next());
    }

    #[test]
    fn exponential_delay() {
        let mut delay = DelayFactoryBuilder::new(DelayFactoryMode::Exponential(100, 2.0))
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
        let mut delay = DelayFactoryBuilder::new(DelayFactoryMode::Constant(500))
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
        let mut delay = DelayFactoryBuilder::new(DelayFactoryMode::Constant(25))
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
