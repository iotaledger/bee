// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) enum Backoff {
    Zero,
    Constant(u64),
    Exponential(u64, f32),
}

impl Iterator for Backoff {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Zero => Some(0),
            Self::Constant(value) => Some(*value),
            Self::Exponential(value, factor) => {
                let prev_value = *value;
                *value = (*value as f32 * *factor) as u64;
                Some(prev_value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_backoff() {
        let mut backoff = Backoff::Zero;

        assert_eq!(Some(0), backoff.next());
        assert_eq!(Some(0), backoff.next());
        assert_eq!(Some(0), backoff.next());
        assert_eq!(Some(0), backoff.next());
    }

    #[test]
    fn constant_backoff() {
        let mut backoff = Backoff::Constant(500);

        assert_eq!(Some(500), backoff.next());
        assert_eq!(Some(500), backoff.next());
        assert_eq!(Some(500), backoff.next());
        assert_eq!(Some(500), backoff.next());
    }

    #[test]
    fn exponential_backoff() {
        let mut backoff = Backoff::Exponential(100, 2.0);

        assert_eq!(Some(100), backoff.next());
        assert_eq!(Some(200), backoff.next());
        assert_eq!(Some(400), backoff.next());
        assert_eq!(Some(800), backoff.next());
    }
}
