use bee_ternary::{Btrit, TritBuf};
use crate::ternary::sponge::batched_curl::mux::BCTrits;

pub struct BCTernaryDemultiplexer {
    bc_trits: BCTrits,
}

impl BCTernaryDemultiplexer {
    pub fn new(bc_trits: BCTrits) -> Self {
        Self { bc_trits }
    }

    pub fn get(&self, index: usize) -> TritBuf {
        let length = self.bc_trits.lo.len();
        let mut result = TritBuf::new();

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

        result
    }
}
