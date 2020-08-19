// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_crypto::ternary::sponge::{BatchHasher, CurlPRounds, BATCH_SIZE};
use bee_ternary::{T1B1Buf, TryteBuf};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn length_243(c: &mut Criterion) {
    let input = "HHPELNTNJIOKLYDUW9NDULWPHCWFRPTDIUWLYUHQWWJVPAKKGKOAZFJPQJBLNDPALCVXGJLRBFSHATF9C";
    let input_trit_buf = TryteBuf::try_from_str(input).unwrap().as_trits().encode::<T1B1Buf>();
    let length = input_trit_buf.len();
    let mut hasher = BatchHasher::new(length, CurlPRounds::Rounds81);

    c.bench_function("batched 243", |b| {
        b.iter(|| {
            for _ in 0..BATCH_SIZE {
                hasher.add(black_box(input_trit_buf.clone()));
            }
            for _ in hasher.hash_batched() {}
        })
    });

    c.bench_function("unbatched 243", |b| {
        b.iter(|| {
            for _ in 0..BATCH_SIZE {
                hasher.add(black_box(input_trit_buf.clone()));
            }
            for _ in hasher.hash_unbatched() {}
        })
    });
}

fn length_8019(c: &mut Criterion) {
    let input = "AQUIC9VCKYJWSGMGHMPGRMHLLCKBOCMLOKMLXAMWSCBEDOWQS9HJBYWNABSUHARMZHQFOMKNBUEKB9YWC9UWQVWIDRFPNUF9QGFORSXLLK9MBVVHSRQMOWEJIKGNMKTKKZLXXKFMSVZDUMMWGYEAWPGXRHJZOYYVOYUFQDELKPTFTFXFGN9KUCLXPSVVX9PXCKEGWBWMMYWVWBXUTAAZZALALFVCJWFP9HKIVGVZSBZSESSIEGTPPLNJZIIXJRRYUREWVOGOCGWMSJUISJHIRNTNCUV9CHRLVFBMCQSVB9DLMPYSJIOBWVJIXEVQORQJHSQLUPNJFGSXAOULRMIIXGVNSPFGFZZTCVHRTXKPHFLAXKKSDXDUTVSWXFIGSYMMSHWWAKTJSCYZHXEBLMMJX9XJUOXGBCZ9RTDPHMRGAHHQNNNK9SNTEFNAMOBPYGIN9OZLJANAWFVUIXZJAJMHJQP9UVANIGSXTFGSANSTXRILDWQOHTUQRXKAPFUA9PDNAEZHTMZIMGNGHESYDXSODNBYVCETFVVBCF9FEPHUSGK9YSFYMDRLBDLLZXQKY9ZXRWVBXGICVNNNCRJLRAWTJYEBJNPSX9ZZLZPF9AISXIOSVVYQVICDLMVKOFUHAKWUKGMPDYJNZCHSNXSFVQPJXWCXN9JBEAGNMWRVTGNOWKADPJ9AJBTVYYVXTFNHTKAYHUGMEBOHYXIKPUYCKNHUAOQUVGWXAEIIT9YWJMCCGIPAQYNUZWWWWRFTF9KRDDDOEFOOJWMRZXZHTPUBH9IKXKGXLUIBFAFLXMIYDXJSAFFGAURXLSDUNZYJYEZLMEGUMQFVKZWBTVRLBDS9RSEQPUSMXLHUBMTYVFMM999VIRVELDUOCDYZRXVQCYB9AJLJYDBIIKCRSQSRQLI9JORWAOFUNEUGEUBNQFMASBWTUTPHJTGFYGHO9PDQDJXZSOC9RRZGRREQY9IYURJGTZRYVCLBGBUVCANFIQWDARHJHUPXJIHZX9GTNOWGJBPVZRYYNWRIHSBPCED9NDWEYUZNHPXVQNNFIUAGABXESBENVWDALPSCLQTXANWTXIGORBGR9XHNDITOLWFZVFZXJCIZGYGDORYIZYALLQETQQMSWEDPYX9EDZKJKDVBNNAK9BHNIVIFMPBN9FJFVFCK9XSMGFDSOXNDELLVYNVVTDDBTYFBMPMMYYDI9LCFFXMHXJQSUFUFASBLIBHWSCFHZWFZOBJOROVMWJ9FSIFJAEVDJZSXEPSLJMNVXYWWIIUJUPYIPVWOBAUSOBCIUAECILFJVNCCPQOZZSIHIWZMXLOXFZUVSXYRWPKCXDTYTHSFHXNDEDC9BREQBOIPECTGMQFEYIJTVXMBFLLJWZJMKMJDOZ9ECKWDOPWPYGV9PQIBTXOWPCTHPVGPVQUBBPSC9NVJLEZOITPJE9ZZNG9KOCMEZEHF9JZZSMPDRRAD9CSV9UVFXTZTTDOCWDYRIWMHAOYUO9SHRKW9MACEO9LPGZBSAWYQVKWMZDUNEE9ONRTDWTZEZVQEZXWHSPFKGAHBIKYBWGYCLZRZBDSVJIUXERPSF9UFQOZSLMMTGS9UCQASYAECTDZFWRBOAWMXOMQFQODFJQJGVOXHNJDLPWSGDDUPBXAOLFFUUYUAOVDMMTIEFTJWMFE9OPANAKKWIMXRHLNHMRZH9TCEONTVCSLADVJYZXWYHDMVBXFOCYFTOXHVZVCELZMUIJZHRALVHZ9NSVIK9VMBGRXX9GOUSGHSERBFIZGC9X9HVWV9VVGEWCPVQI9CFRGAYVPSVULWQNKTCJUZBYYVPBNHSTIMXVZOCUVRYJIRCT9LFANYATSAPPDFORPAYWXWFG9CECXRKTV9PMQSZMOVVYYKX9JBAVRYSCMXWFM9QVS9QUUPKNJSUSOMYCUIOOAAD9NHZZKOYMMZNSQYSDYBBYMRTEWQUYUHFLVHUUFWIXQRAWSPZPKETGKJSFGMAKFMVSTQEDLTUJYQONNBWVJDHTLVIGOSKKPQPDSHYUAAFAOVMOAXOMRWOBTAOIAGVBRTFELKIFZNFSADZYSGHBWLTOEJCUDFVRPPLWDMHSOJBWEBRTCRIEEMDSEKKXPRZGGIJYHOWYQKSJH9MFCHYKBGXUWYXQGJRXFLCDHVV9TKDITLCAIRZTJPACOPINMUC9RLFTYVALBRVA9OBMOYMFHQWC9TAISJODUMBIWC9RSPSVZGAJOZXTULEOPTKCGYYOOKDEOSKLDEGDXRQHIEMGZUXMOW999999999999999999999999999GD9999999999999999999999999JAYNICD99999999999B99999999YGOQEYWBEGZJGLJSPPFFVWA9RIDDZRFEGDZRZGATZTKSETYBXQHKQZGYLJVOCIQEDVCWAWBK9TUPK9DIWCEWQXNQCKQQEAAOFSEUGCGPMXCIBPCR9ASZEDDPRGWBLOVDSKMDCAUBYFLDTRJRUCAPCWSSSVILNZ9999UQJFWFRNIWMVPSICVZSWBW9JMEVVDHQRJWOPVHRBDMNEGRCVNYRLKPHZLQMLVYYBFVUXAIXYRRLW99999999999999999999999999999999USHXKUSRF999999999K99999999RC999999999PTA9999999999999";
    let input_trit_buf = TryteBuf::try_from_str(input).unwrap().as_trits().encode::<T1B1Buf>();
    let length = input_trit_buf.len();
    let mut hasher = BatchHasher::new(length, CurlPRounds::Rounds81);

    c.bench_function("batched 8019", |b| {
        b.iter(|| {
            for _ in 0..BATCH_SIZE {
                hasher.add(black_box(input_trit_buf.clone()));
            }
            for _ in hasher.hash_batched() {}
        })
    });

    c.bench_function("unbatched 8019", |b| {
        b.iter(|| {
            for _ in 0..BATCH_SIZE {
                hasher.add(black_box(input_trit_buf.clone()));
            }
            for _ in hasher.hash_unbatched() {}
        })
    });
}

criterion_group!(benches, length_243, length_8019);
criterion_main!(benches);
