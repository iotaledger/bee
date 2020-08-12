pub mod bct_curl;
pub mod mux;

use bee_ternary::TritBuf;

pub struct BatchHasher {
    hash_length: usize,
    rounds: usize,
}

impl BatchHasher {
    pub fn new(hash_length: usize, rounds: usize) -> Self {
        Self { hash_length, rounds }
    }

    pub fn process(&self, inputs: Vec<TritBuf>) -> Vec<TritBuf> {
        let n_inputs = inputs.len();
        if n_inputs > 1 {
            let mut multiplexer = mux::multiplexer::BCTernaryMultiplexer::new();

            for hash in inputs {
                multiplexer.add(hash);
            }

            let bc_trits = multiplexer.extract();

            let mut bct_curl = bct_curl::BCTCurl::new(self.hash_length, self.rounds, 8 * std::mem::size_of::<usize>());

            bct_curl.reset();
            bct_curl.absorb(bc_trits);

            let bc_trits = bct_curl.squeeze(243);

            let demux = mux::demultiplexer::BCTernaryDemultiplexer::new(bc_trits);

            (0..n_inputs).map(|i| demux.get(i)).collect()
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bee_ternary::{T1B1Buf, T3B1Buf, TryteBuf};

    #[test]
    fn foo() {
        let input  = "HHPELNTNJIOKLYDUW9NDULWPHCWFRPTDIUWLYUHQWWJVPAKKGKOAZFJPQJBLNDPALCVXGJLRBFSHATF9C";
        let output = "XMJNTUHSDIVRPCBKWPOZWTJFDRSKXZKNJEJSBIHQXQE9YVGTOFF9DMQTFOLCPVLPGMYUGJSTOZZSIGFGH";

        let input_trit_buf = TryteBuf::try_from_str(input).unwrap().as_trits().encode::<T1B1Buf>();

        let expected_hash = TryteBuf::try_from_str(output).unwrap().as_trits().encode::<T1B1Buf>();

        let batch_hasher = BatchHasher::new(input_trit_buf.len(), 81);

        let hashes = batch_hasher.process(vec![input_trit_buf; 64]);

        for (index, hash) in hashes.iter().enumerate() {
            assert_eq!(hash, &hashes[0], "input {} failed", index);
        }

        assert_eq!(hashes[0], expected_hash);
    }
}
