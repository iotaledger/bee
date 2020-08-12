use crate::ternary::sponge::batched_curl::mux::{BCTrit, BCTrits};
use bee_ternary::{Btrit, TritBuf};

pub struct BCTernaryMultiplexer {
    trinaries: Vec<TritBuf>,
}

impl BCTernaryMultiplexer {
    pub fn new() -> Self {
        Self { trinaries: Vec::new() }
    }

    pub fn add(&mut self, trits: TritBuf) -> usize {
        self.trinaries.push(trits);
        self.trinaries.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&TritBuf> {
        self.trinaries.get(index)
    }

    pub fn extract(&mut self) -> BCTrits {
        let trinaries_count = self.trinaries.len();
        let trits_count = self.trinaries[0].len();

        let mut result = BCTrits {
            lo: vec![0; trits_count],
            hi: vec![0; trits_count],
        };

        for i in 0..trits_count {
            let mut bc_trit = BCTrit { lo: 0, hi: 0 };

            for j in 0..trinaries_count {
                match self.trinaries[j][i] {
                    Btrit::NegOne => bc_trit.lo |= 1 << j,
                    Btrit::PlusOne => bc_trit.hi |= 1 << j,
                    Btrit::Zero => {
                        bc_trit.lo |= 1 << j;
                        bc_trit.hi |= 1 << j;
                    }
                }
            }

            result.lo[i] = bc_trit.lo;
            result.hi[i] = bc_trit.hi;
        }
        result
    }
}
