#![allow(missing_docs)]

mod bct_curl;
mod bct;

pub const BATCH_SIZE: usize = 8 * std::mem::size_of::<usize>();
const HIGH_BITS: usize = usize::max_value();

use bct_curl::BCTCurl;
use bee_ternary::{Btrit, TritBuf};
use bct::BCTritBuf;

use crate::ternary::sponge::{CurlP, CurlPRounds, Sponge};

pub struct BatchHasher {
    trits: Vec<TritBuf>,
    inputs: BCTritBuf,
    hashes: BCTritBuf,
    bct_curl: BCTCurl,
    curl: CurlP,
}

impl BatchHasher {
    pub fn new(input_length: usize, hash_length: usize, rounds: CurlPRounds) -> Self {
        Self {
            trits: Vec::with_capacity(BATCH_SIZE),
            inputs: BCTritBuf::zeros(input_length),
            hashes: BCTritBuf::zeros(hash_length),
            bct_curl: BCTCurl::new(hash_length, rounds as usize),
            curl: CurlP::new(rounds),
        }
    }

    pub fn add(&mut self, input: TritBuf) {
        self.trits.push(input);
    }

    fn mux(&mut self) {
        let count = self.trits.len();
        for i in 0..self.inputs.len() {
            let bc_trit = self.inputs.get_mut(i);

            for j in 0..count {
                match self.trits[j][i] {
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

    fn demux(&self, index: usize) -> TritBuf {
        let length = self.hashes.len();
        let mut result = Vec::with_capacity(length);

        for i in 0..length {
            let low = (self.hashes.lo()[i] >> index) & 1;
            let hi = (self.hashes.hi()[i] >> index) & 1;

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

    pub fn hash_batched<'a>(&'a mut self) -> impl Iterator<Item = TritBuf> + 'a {
        let total = self.trits.len();
        self.hashes.fill(0);
        self.bct_curl.reset();
        self.mux();
        self.bct_curl.absorb(&self.inputs);
        self.bct_curl.squeeze_into(&mut self.hashes);
        self.trits.clear();
        self.inputs.fill(0);
        BatchedHashes {
            hasher: self,
            range: 0..total,
        }
    }

    pub fn hash_unbatched<'a>(&'a mut self) -> impl Iterator<Item = TritBuf> + 'a {
        UnbatchedHashes {
            curl: &mut self.curl,
            trits: self.trits.drain(..),
        }
    }
}

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

struct UnbatchedHashes<'a> {
    curl: &'a mut CurlP,
    trits: std::vec::Drain<'a, TritBuf>,
}

impl<'a> Iterator for UnbatchedHashes<'a> {
    type Item = TritBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.trits.next()?;
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
