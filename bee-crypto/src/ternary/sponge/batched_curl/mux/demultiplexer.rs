use crate::ternary::sponge::batched_curl::mux::BCTritBuf;
use bee_ternary::{Btrit, TritBuf};

pub struct BCTernaryDemultiplexer {
    bc_trits: BCTritBuf,
}

impl BCTernaryDemultiplexer {
    pub fn new(bc_trits: BCTritBuf) -> Self {
        Self { bc_trits }
    }

    pub fn get(&self, index: usize) -> TritBuf {
        let length = self.bc_trits.lo.len();
        let mut result = Vec::with_capacity(length);

        for i in 0..length {
            let low = (self.bc_trits.lo[i] >> index) & 1;
            let hi = (self.bc_trits.hi[i] >> index) & 1;

            let trit = match (low, hi) {
                (1, 0) => Btrit::NegOne,
                (0, 1) => Btrit::PlusOne,
                (1, 1) => Btrit::Zero,
                _ => Btrit::Zero,
            };

            result.push(trit);
        }

        TritBuf::from_trits(&result)
    }
}
