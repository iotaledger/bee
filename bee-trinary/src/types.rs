//! Meaningful type aliases and a signed 129 bit integer.

use std::ops::AddAssign;

/// Representation of an unsigned byte, which is mostly there to explicitly distinguish it
/// from `Trit`s and `Tryte`s.
pub type Byte = u8;

/// Representation of a trit.
pub type Trit = i8;

/// Representation of a tryte.
pub type Tryte = u8;

/// The sign for the `I129` signed 129 bit integer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Sign {
    /// Positive sign.
    Pos,
    /// Negative sign.
    Neg,
}

/// A signed 129 bit integer to represent any positive or negative integer that can be
/// encoded with up to 27 (balanced trinary) trytes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct I129(pub Sign, pub u128);

impl I129 {
    /// Creates a I129 representing the tuple (+, 0).
    pub fn zero() -> Self {
        I129(Sign::Pos, 0)
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl AddAssign for I129 {
    fn add_assign(&mut self, other: Self) {
        match (&self.0, other.0) {
            (Sign::Pos, Sign::Neg) => {
                if other.1 > self.1 {
                    self.0 = Sign::Neg;
                    self.1 = other.1 - self.1;
                } else {
                    self.1 -= other.1;
                }
            }
            (Sign::Neg, Sign::Pos) => {
                if other.1 >= self.1 {
                    self.0 = Sign::Pos;
                    self.1 = other.1 - self.1;
                } else {
                    self.1 -= other.1;
                }
            }
            (_, _) => self.1 += other.1,
        }
    }
}

impl From<u128> for I129 {
    fn from(n: u128) -> Self {
        Self(Sign::Pos, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use test::Bencher;

    const I129_MAX: u128 = 1_000_000;

    #[test]
    fn bench_i129_add_assign_with_instant() {
        let mut v = I129::from(0);
        let start = Instant::now();
        for i in 1..=I129_MAX {
            v += I129(Sign::Pos, i);
        }
        let elaps = start.elapsed();

        println!("result={}, {} ns", v.1, elaps.subsec_nanos());

        assert_eq!(I129(Sign::Pos, 500_000_500_000), v);
    }

    #[bench]
    fn bench_i129_add_assign_with_bencher(b: &mut Bencher) {
        b.iter(|| {
            let mut v = I129::from(0);
            for i in 1..=I129_MAX {
                v += I129(Sign::Pos, i);
            }
            assert_eq!(I129(Sign::Pos, 500_000_500_000), v);
        })
    }

    #[test]
    fn test_i129_add_assign() {
        // Example: (+, 2) + (-, 4) = (-, 2)
        let mut a = I129(Sign::Pos, 2);
        let b = I129(Sign::Neg, 4);
        a += b;
        assert_eq!(I129(Sign::Neg, 2), a);

        // Example: (+, 2) + (-, 1) = (+, 1)
        let mut a = I129(Sign::Pos, 2);
        let b = I129(Sign::Neg, 1);
        a += b;
        assert_eq!(I129(Sign::Pos, 1), a);

        // Example: (+, 2) + (-, 2) = (+, 0)
        let mut a = I129(Sign::Pos, 2);
        let b = I129(Sign::Neg, 2);
        a += b;
        assert_eq!(I129(Sign::Pos, 0), a);

        // Example: (-, 2) + (+, 4) = (+, 2)
        let mut a = I129(Sign::Neg, 2);
        let b = I129(Sign::Pos, 4);
        a += b;
        assert_eq!(I129(Sign::Pos, 2), a);

        // Example: (-, 2) + (+, 1) = (-, 1)
        let mut a = I129(Sign::Neg, 2);
        let b = I129(Sign::Pos, 1);
        a += b;
        assert_eq!(I129(Sign::Neg, 1), a);

        // Example: (-, 2) + (+, 2) = (+, 0)
        let mut a = I129(Sign::Neg, 2);
        let b = I129(Sign::Pos, 2);
        a += b;
        assert_eq!(I129(Sign::Pos, 0), a);

        // Example: (+, 2) + (+, 4) = (+, 6)
        let mut a = I129(Sign::Pos, 2);
        let b = I129(Sign::Pos, 4);
        a += b;
        assert_eq!(I129(Sign::Pos, 6), a);

        // Example: (-, 2) + (-, 4) = (-, 6)
        let mut a = I129(Sign::Neg, 2);
        let b = I129(Sign::Neg, 4);
        a += b;
        assert_eq!(I129(Sign::Neg, 6), a);
    }
}
