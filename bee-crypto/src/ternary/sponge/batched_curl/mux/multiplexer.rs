use crate::ternary::sponge::batched_curl::{mux::BCTrits, BATCH_SIZE};
use bee_ternary::{Btrit, TritBuf};

pub struct BCTernaryMultiplexer<'a> {
    trinaries: &'a [TritBuf; BATCH_SIZE],
}

impl<'a> BCTernaryMultiplexer<'a> {
    pub fn new(trinaries: &'a [TritBuf]) -> Self {
        assert_eq!(trinaries.len(), BATCH_SIZE);
        let ptr = trinaries.as_ptr() as *const [TritBuf; BATCH_SIZE];
        Self {
            trinaries: unsafe { &*ptr },
        }
    }

    pub fn extract(&mut self) -> BCTrits {
        let trits_count = self.trinaries[0].len();

        let mut result = BCTrits {
            lo: vec![0; trits_count],
            hi: vec![0; trits_count],
        };

        for i in 0..trits_count {
            let bc_trit_lo = &mut result.lo[i];
            let bc_trit_hi = &mut result.hi[i];

            for j in 0..BATCH_SIZE {
                match self.trinaries[j][i] {
                    Btrit::NegOne => *bc_trit_lo |= 1 << j,
                    Btrit::PlusOne => *bc_trit_hi |= 1 << j,
                    Btrit::Zero => {
                        *bc_trit_lo |= 1 << j;
                        *bc_trit_hi |= 1 << j;
                    }
                }
            }
        }
        result
    }
}
