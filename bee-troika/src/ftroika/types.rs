use crate::types::Trit;

use core::fmt;

#[derive(Clone, Copy)]
pub struct T27 {
    pub p: u32,
    pub n: u32,
}

impl fmt::Debug for T27 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T27: [p: {}, n: {}]", self.p, self.n,)
    }
}

impl T27 {
    pub fn new(p: u32, n: u32) -> T27 {
        T27 { p: p, n: n }
    }

    pub fn clean(&self) -> T27 {
        T27::new(self.p & 0x07ffffffu32, self.n & 0x07ffffffu32)
    }

    pub fn add(&self, other: &T27) -> T27 {
        let self_zero: u32 = !self.p & !self.n;
        let p = !(self.n ^ other.n) & !(self_zero ^ other.p);
        let n = !(self.p ^ other.p) & !(self_zero ^ other.n);
        T27::new(p, n)
    }

    pub fn mul(&self, other: &T27) -> T27 {
        let p = (self.p & other.p) | (self.n & other.n);
        let n = (self.p & other.n) | (self.n & other.p);
        T27::new(p, n)
    }

    pub fn zero() -> T27 {
        T27::new(0, 0)
    }

    /*
    fn one() -> T27 {
        T27::new(0x07ffffffu32, 0)
    }
    */

    fn minus() -> T27 {
        T27::new(0, 0x07ffffffu32)
    }

    pub fn dec(&self) -> T27 {
        T27::minus().add(&self)
    }

    /*
    fn inc(&self) -> T27 {
        T27::one().add(&self)
    }
    */

    pub fn set(&mut self, pos: usize, value: Trit) {
        let mask: u32 = 1u32 << pos;
        //self.p &= !mask;
        //self.n &= !mask;
        match value {
            1 => self.p |= mask,
            2 => self.n |= mask,
            _ => (),
        }
    }

    pub fn get(&mut self, pos: usize) -> Trit {
        let mask: u32 = 1u32 << pos;
        if self.p & mask != 0 {
            return 1;
        } else if self.n & mask != 0 {
            return 2;
        }
        0
    }

    pub fn roll(&self, by: usize) -> T27 {
        let p = ((self.p << by) | (self.p >> (27 - by))) & 0x07ffffff;
        let n = ((self.n << by) | (self.n >> (27 - by))) & 0x07ffffff;
        T27::new(p, n)
    }
}
