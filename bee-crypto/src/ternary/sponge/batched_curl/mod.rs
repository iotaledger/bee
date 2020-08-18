mod bct;
mod bct_curl;

use bee_ternary::{Btrit, TritBuf};

use crate::ternary::sponge::{CurlP, CurlPRounds, Sponge};
use bct::BCTritBuf;
use bct_curl::BCTCurl;

/// The number of inputs that can be processed in a single batch.
pub const BATCH_SIZE: usize = 8 * std::mem::size_of::<usize>();
const HIGH_BITS: usize = usize::max_value();
/// A hasher that can process several inputs at the same time in batches.
///
/// This hasher works by interleaving the trits of the inputs in each batch and hashing this
/// interleaved representation. It is also able to fall back to the regular CurlP algorithm if
/// required.
pub struct BatchHasher {
    /// The trits of the inputs before being interleaved.
    trit_inputs: Vec<TritBuf>,
    /// An interleaved representation of the input trits.
    bct_inputs: BCTritBuf,
    /// An interleaved representation of the output trits.
    bct_hashes: BCTritBuf,
    /// The CurlP hasher for binary coded trits.
    bct_curl: BCTCurl,
    /// The regular CurlP hasher.
    curl: CurlP,
}

impl BatchHasher {
    /// Create a new hasher.
    ///
    /// It requires the length of the input, the length of the output hash and the number of
    /// rounds.
    pub fn new(input_length: usize, hash_length: usize, rounds: CurlPRounds) -> Self {
        Self {
            trit_inputs: Vec::with_capacity(BATCH_SIZE),
            bct_inputs: BCTritBuf::zeros(input_length),
            bct_hashes: BCTritBuf::zeros(hash_length),
            bct_curl: BCTCurl::new(hash_length, rounds as usize),
            curl: CurlP::new(rounds),
        }
    }
    /// Add a new input to the batch.
    ///
    /// It panics if the size of the batch exceeds `BATCH_SIZE`.
    pub fn add(&mut self, input: TritBuf) {
        assert!(self.trit_inputs.len() <= BATCH_SIZE);
        self.trit_inputs.push(input);
    }
    /// Multiplex or interleave the input trits in the bash.
    ///
    /// Before doing the actual interleaving, each trit is encoded as two bits which are usually
    /// refered as the low and high bit.
    ///
    /// | Trit | Low bit | High bit |
    /// |------|---------|----------|
    /// |  -1  |    1    |     0    |
    /// |   0  |    0    |     1    |
    /// |   1  |    1    |     1    |
    ///
    /// Then the low and high bits are interleaved into two vectors of integers. Each integer has
    /// size `BATCH_SIZE` and there are `input_length` integers in each vector.  This means that
    /// the low and high bits of the transaction number `N` in the batch are stored in the position
    /// `N` of each integer.
    ///
    /// This step works correctly even if there are less than `BATCH_SIZE` inputs.
    fn mux(&mut self) {
        let count = self.trit_inputs.len();
        for i in 0..self.bct_inputs.len() {
            let bc_trit = self.bct_inputs.get_mut(i);

            for j in 0..count {
                match self.trit_inputs[j][i] {
                    Btrit::NegOne => *bc_trit.lo |= 1 << j,
                    Btrit::PlusOne => *bc_trit.hi |= 1 << j,
                    Btrit::Zero => {
                        *bc_trit.lo |= 1 << j;
                        *bc_trit.hi |= 1 << j;
                    }
                }
            }
        }
    }
    /// Demultiplex the bits of the output to obtain the hash of the input with an specific index.
    ///
    /// This is the inverse of the `mux` function, but it is applied over the vector with the
    /// binary encoding of the output hashes. Each pair of low and high bits in the `bct_hashes`
    /// field is decoded into a trit using the same convention as the `mux` step with an additional
    /// rule for the `(0, 0)` pair of bits which is mapped to the `0` trit.
    fn demux(&self, index: usize) -> TritBuf {
        let length = self.bct_hashes.len();
        let mut result = Vec::with_capacity(length);

        for i in 0..length {
            let low = (self.bct_hashes.lo()[i] >> index) & 1;
            let hi = (self.bct_hashes.hi()[i] >> index) & 1;

            let trit = match (low, hi) {
                (1, 0) => Btrit::NegOne,
                (0, 1) => Btrit::PlusOne,
                (1, 1) => Btrit::Zero,
                // This can only be `(0, 0)`.
                _ => Btrit::Zero,
            };

            result.push(trit);
        }

        TritBuf::from_trits(&result)
    }
    /// Hash the received inputs using the batched version of CurlP.
    ///
    /// This function also takes care of cleaning all the buffers of the struct and resetting the
    /// batched CurlP hasher so it can be called at any time.
    pub fn hash_batched<'a>(&'a mut self) -> impl Iterator<Item = TritBuf> + 'a {
        let total = self.trit_inputs.len();
        // Fill the `hashes` buffer with zeros.
        self.bct_hashes.fill(0);
        // Reset batched CurlP hasher.
        self.bct_curl.reset();
        // Multiplex the trits in `trits` and dump them into `inputs`
        self.mux();
        // Do the regular sponge steps.
        self.bct_curl.absorb(&self.bct_inputs);
        self.bct_curl.squeeze_into(&mut self.bct_hashes);
        // Clear the `trits` buffer to allow receiving a new batch.
        self.trit_inputs.clear();
        // Fill the `inputs` buffer with zeros.
        self.bct_inputs.fill(0);
        // Return an iterator for the output hashes.
        BatchedHashes {
            hasher: self,
            range: 0..total,
        }
    }
    /// Hash the received inputs using the regular version of CurlP.
    ///
    /// In particular this function does not use the `bct_inputs` and `bct_hashes` buffers, takes
    /// care of cleaning the `trit_inputs` buffer and resets the regular CurlP hasher so it can be
    /// called at any time.
    pub fn hash_unbatched<'a>(&'a mut self) -> impl Iterator<Item = TritBuf> + 'a {
        self.curl.reset();
        UnbatchedHashes {
            curl: &mut self.curl,
            trit_inputs: self.trit_inputs.drain(..),
        }
    }
}

/// A helper iterator type for the output of the `hash_batched` method.
struct BatchedHashes<'a> {
    hasher: &'a mut BatchHasher,
    range: std::ops::Range<usize>,
}

impl<'a> Iterator for BatchedHashes<'a> {
    type Item = TritBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;
        Some(self.hasher.demux(index))
    }
}

/// A helper iterator type for the output of the `hash_unbatched` method.
struct UnbatchedHashes<'a> {
    curl: &'a mut CurlP,
    trit_inputs: std::vec::Drain<'a, TritBuf>,
}

impl<'a> Iterator for UnbatchedHashes<'a> {
    type Item = TritBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.trit_inputs.next()?;
        Some(self.curl.digest(&buf).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bee_ternary::{T1B1Buf, TryteBuf};

    #[test]
    fn foo() {
        let input = "HHPELNTNJIOKLYDUW9NDULWPHCWFRPTDIUWLYUHQWWJVPAKKGKOAZFJPQJBLNDPALCVXGJLRBFSHATF9C";
        let output = "XMJNTUHSDIVRPCBKWPOZWTJFDRSKXZKNJEJSBIHQXQE9YVGTOFF9DMQTFOLCPVLPGMYUGJSTOZZSIGFGH";

        let input_trit_buf = TryteBuf::try_from_str(input).unwrap().as_trits().encode::<T1B1Buf>();

        let expected_hash = TryteBuf::try_from_str(output).unwrap().as_trits().encode::<T1B1Buf>();

        let mut batch_hasher = BatchHasher::new(input_trit_buf.len(), expected_hash.len(), CurlPRounds::Rounds81);

        for _ in 0..64 {
            batch_hasher.add(input_trit_buf.clone());
        }

        for (index, hash) in batch_hasher.hash_batched().enumerate() {
            assert_eq!(expected_hash, hash, "input {} failed", index);
        }
    }

    #[test]
    fn bar() {
        let input = "AQUIC9VCKYJWSGMGHMPGRMHLLCKBOCMLOKMLXAMWSCBEDOWQS9HJBYWNABSUHARMZHQFOMKNBUEKB9YWC9UWQVWIDRFPNUF9QGFORSXLLK9MBVVHSRQMOWEJIKGNMKTKKZLXXKFMSVZDUMMWGYEAWPGXRHJZOYYVOYUFQDELKPTFTFXFGN9KUCLXPSVVX9PXCKEGWBWMMYWVWBXUTAAZZALALFVCJWFP9HKIVGVZSBZSESSIEGTPPLNJZIIXJRRYUREWVOGOCGWMSJUISJHIRNTNCUV9CHRLVFBMCQSVB9DLMPYSJIOBWVJIXEVQORQJHSQLUPNJFGSXAOULRMIIXGVNSPFGFZZTCVHRTXKPHFLAXKKSDXDUTVSWXFIGSYMMSHWWAKTJSCYZHXEBLMMJX9XJUOXGBCZ9RTDPHMRGAHHQNNNK9SNTEFNAMOBPYGIN9OZLJANAWFVUIXZJAJMHJQP9UVANIGSXTFGSANSTXRILDWQOHTUQRXKAPFUA9PDNAEZHTMZIMGNGHESYDXSODNBYVCETFVVBCF9FEPHUSGK9YSFYMDRLBDLLZXQKY9ZXRWVBXGICVNNNCRJLRAWTJYEBJNPSX9ZZLZPF9AISXIOSVVYQVICDLMVKOFUHAKWUKGMPDYJNZCHSNXSFVQPJXWCXN9JBEAGNMWRVTGNOWKADPJ9AJBTVYYVXTFNHTKAYHUGMEBOHYXIKPUYCKNHUAOQUVGWXAEIIT9YWJMCCGIPAQYNUZWWWWRFTF9KRDDDOEFOOJWMRZXZHTPUBH9IKXKGXLUIBFAFLXMIYDXJSAFFGAURXLSDUNZYJYEZLMEGUMQFVKZWBTVRLBDS9RSEQPUSMXLHUBMTYVFMM999VIRVELDUOCDYZRXVQCYB9AJLJYDBIIKCRSQSRQLI9JORWAOFUNEUGEUBNQFMASBWTUTPHJTGFYGHO9PDQDJXZSOC9RRZGRREQY9IYURJGTZRYVCLBGBUVCANFIQWDARHJHUPXJIHZX9GTNOWGJBPVZRYYNWRIHSBPCED9NDWEYUZNHPXVQNNFIUAGABXESBENVWDALPSCLQTXANWTXIGORBGR9XHNDITOLWFZVFZXJCIZGYGDORYIZYALLQETQQMSWEDPYX9EDZKJKDVBNNAK9BHNIVIFMPBN9FJFVFCK9XSMGFDSOXNDELLVYNVVTDDBTYFBMPMMYYDI9LCFFXMHXJQSUFUFASBLIBHWSCFHZWFZOBJOROVMWJ9FSIFJAEVDJZSXEPSLJMNVXYWWIIUJUPYIPVWOBAUSOBCIUAECILFJVNCCPQOZZSIHIWZMXLOXFZUVSXYRWPKCXDTYTHSFHXNDEDC9BREQBOIPECTGMQFEYIJTVXMBFLLJWZJMKMJDOZ9ECKWDOPWPYGV9PQIBTXOWPCTHPVGPVQUBBPSC9NVJLEZOITPJE9ZZNG9KOCMEZEHF9JZZSMPDRRAD9CSV9UVFXTZTTDOCWDYRIWMHAOYUO9SHRKW9MACEO9LPGZBSAWYQVKWMZDUNEE9ONRTDWTZEZVQEZXWHSPFKGAHBIKYBWGYCLZRZBDSVJIUXERPSF9UFQOZSLMMTGS9UCQASYAECTDZFWRBOAWMXOMQFQODFJQJGVOXHNJDLPWSGDDUPBXAOLFFUUYUAOVDMMTIEFTJWMFE9OPANAKKWIMXRHLNHMRZH9TCEONTVCSLADVJYZXWYHDMVBXFOCYFTOXHVZVCELZMUIJZHRALVHZ9NSVIK9VMBGRXX9GOUSGHSERBFIZGC9X9HVWV9VVGEWCPVQI9CFRGAYVPSVULWQNKTCJUZBYYVPBNHSTIMXVZOCUVRYJIRCT9LFANYATSAPPDFORPAYWXWFG9CECXRKTV9PMQSZMOVVYYKX9JBAVRYSCMXWFM9QVS9QUUPKNJSUSOMYCUIOOAAD9NHZZKOYMMZNSQYSDYBBYMRTEWQUYUHFLVHUUFWIXQRAWSPZPKETGKJSFGMAKFMVSTQEDLTUJYQONNBWVJDHTLVIGOSKKPQPDSHYUAAFAOVMOAXOMRWOBTAOIAGVBRTFELKIFZNFSADZYSGHBWLTOEJCUDFVRPPLWDMHSOJBWEBRTCRIEEMDSEKKXPRZGGIJYHOWYQKSJH9MFCHYKBGXUWYXQGJRXFLCDHVV9TKDITLCAIRZTJPACOPINMUC9RLFTYVALBRVA9OBMOYMFHQWC9TAISJODUMBIWC9RSPSVZGAJOZXTULEOPTKCGYYOOKDEOSKLDEGDXRQHIEMGZUXMOW999999999999999999999999999GD9999999999999999999999999JAYNICD99999999999B99999999YGOQEYWBEGZJGLJSPPFFVWA9RIDDZRFEGDZRZGATZTKSETYBXQHKQZGYLJVOCIQEDVCWAWBK9TUPK9DIWCEWQXNQCKQQEAAOFSEUGCGPMXCIBPCR9ASZEDDPRGWBLOVDSKMDCAUBYFLDTRJRUCAPCWSSSVILNZ9999UQJFWFRNIWMVPSICVZSWBW9JMEVVDHQRJWOPVHRBDMNEGRCVNYRLKPHZLQMLVYYBFVUXAIXYRRLW99999999999999999999999999999999USHXKUSRF999999999K99999999RC999999999PTA9999999999999";
        let output = "DZTGVIDBLFLMPMWRHINVCLSXZBOCNRMSFAZOZFLEYGOWJQCXGJTGUCK9YM9KWRZEOSBWBWLTDOYRZ9999";

        let input_trit_buf = TryteBuf::try_from_str(input).unwrap().as_trits().encode::<T1B1Buf>();

        let expected_hash = TryteBuf::try_from_str(output).unwrap().as_trits().encode::<T1B1Buf>();

        let mut batch_hasher = BatchHasher::new(input_trit_buf.len(), expected_hash.len(), CurlPRounds::Rounds81);

        for _ in 0..64 {
            batch_hasher.add(input_trit_buf.clone());
        }

        for (index, hash) in batch_hasher.hash_batched().enumerate() {
            assert_eq!(expected_hash, hash, "input {} failed", index);
        }
    }
}
