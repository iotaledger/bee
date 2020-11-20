// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_crypto::ternary::sponge::{BatchHasher, CurlPRounds, BATCH_SIZE};
use bee_ternary::{T1B1Buf, T5B1Buf, TritBuf, TryteBuf};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn batched_hasher_t5b1(input: &TritBuf<T5B1Buf>) {
    let mut hasher = BatchHasher::new(input.len(), CurlPRounds::Rounds81);

    for _ in 0..BATCH_SIZE {
        hasher.add(input.clone());
    }
    for _ in hasher.hash_batched() {}
}

fn unbatched_hasher_t5b1(input: &TritBuf<T5B1Buf>) {
    let mut hasher = BatchHasher::new(input.len(), CurlPRounds::Rounds81);

    for _ in 0..BATCH_SIZE {
        hasher.add(input.clone());
    }
    for _ in hasher.hash_unbatched() {}
}

fn batched_hasher_encoding_t5b1(input: &TritBuf<T5B1Buf>) {
    let mut hasher = BatchHasher::new(input.len(), CurlPRounds::Rounds81);

    for _ in 0..BATCH_SIZE {
        hasher.add(input.encode::<T1B1Buf>());
    }
    for _ in hasher.hash_batched() {}
}

fn unbatched_hasher_encoding_t5b1(input: &TritBuf<T5B1Buf>) {
    let mut hasher = BatchHasher::new(input.len(), CurlPRounds::Rounds81);

    for _ in 0..BATCH_SIZE {
        hasher.add(input.encode::<T1B1Buf>());
    }
    for _ in hasher.hash_unbatched() {}
}

fn batched_hasher_t1b1(input: &TritBuf<T1B1Buf>) {
    let mut hasher = BatchHasher::new(input.len(), CurlPRounds::Rounds81);

    for _ in 0..BATCH_SIZE {
        hasher.add(input.clone());
    }
    for _ in hasher.hash_batched() {}
}

fn unbatched_hasher_t1b1(input: &TritBuf<T1B1Buf>) {
    let mut hasher = BatchHasher::new(input.len(), CurlPRounds::Rounds81);

    for _ in 0..BATCH_SIZE {
        hasher.add(input.clone());
    }
    for _ in hasher.hash_unbatched() {}
}

fn bench_hasher(c: &mut Criterion) {
    let input_243 = "HHPELNTNJIOKLYDUW9NDULWPHCWFRPTDIUWLYUHQWWJVPAKKGKOAZFJPQJBLNDPALCVXGJLRBFSHATF9C";
    let input_8019 = "AQUIC9VCKYJWSGMGHMPGRMHLLCKBOCMLOKMLXAMWSCBEDOWQS9HJBYWNABSUHARMZHQFOMKNBUEKB9YWC9UWQVWIDRFPNUF9QGFORSXLLK9MBVVHSRQMOWEJIKGNMKTKKZLXXKFMSVZDUMMWGYEAWPGXRHJZOYYVOYUFQDELKPTFTFXFGN9KUCLXPSVVX9PXCKEGWBWMMYWVWBXUTAAZZALALFVCJWFP9HKIVGVZSBZSESSIEGTPPLNJZIIXJRRYUREWVOGOCGWMSJUISJHIRNTNCUV9CHRLVFBMCQSVB9DLMPYSJIOBWVJIXEVQORQJHSQLUPNJFGSXAOULRMIIXGVNSPFGFZZTCVHRTXKPHFLAXKKSDXDUTVSWXFIGSYMMSHWWAKTJSCYZHXEBLMMJX9XJUOXGBCZ9RTDPHMRGAHHQNNNK9SNTEFNAMOBPYGIN9OZLJANAWFVUIXZJAJMHJQP9UVANIGSXTFGSANSTXRILDWQOHTUQRXKAPFUA9PDNAEZHTMZIMGNGHESYDXSODNBYVCETFVVBCF9FEPHUSGK9YSFYMDRLBDLLZXQKY9ZXRWVBXGICVNNNCRJLRAWTJYEBJNPSX9ZZLZPF9AISXIOSVVYQVICDLMVKOFUHAKWUKGMPDYJNZCHSNXSFVQPJXWCXN9JBEAGNMWRVTGNOWKADPJ9AJBTVYYVXTFNHTKAYHUGMEBOHYXIKPUYCKNHUAOQUVGWXAEIIT9YWJMCCGIPAQYNUZWWWWRFTF9KRDDDOEFOOJWMRZXZHTPUBH9IKXKGXLUIBFAFLXMIYDXJSAFFGAURXLSDUNZYJYEZLMEGUMQFVKZWBTVRLBDS9RSEQPUSMXLHUBMTYVFMM999VIRVELDUOCDYZRXVQCYB9AJLJYDBIIKCRSQSRQLI9JORWAOFUNEUGEUBNQFMASBWTUTPHJTGFYGHO9PDQDJXZSOC9RRZGRREQY9IYURJGTZRYVCLBGBUVCANFIQWDARHJHUPXJIHZX9GTNOWGJBPVZRYYNWRIHSBPCED9NDWEYUZNHPXVQNNFIUAGABXESBENVWDALPSCLQTXANWTXIGORBGR9XHNDITOLWFZVFZXJCIZGYGDORYIZYALLQETQQMSWEDPYX9EDZKJKDVBNNAK9BHNIVIFMPBN9FJFVFCK9XSMGFDSOXNDELLVYNVVTDDBTYFBMPMMYYDI9LCFFXMHXJQSUFUFASBLIBHWSCFHZWFZOBJOROVMWJ9FSIFJAEVDJZSXEPSLJMNVXYWWIIUJUPYIPVWOBAUSOBCIUAECILFJVNCCPQOZZSIHIWZMXLOXFZUVSXYRWPKCXDTYTHSFHXNDEDC9BREQBOIPECTGMQFEYIJTVXMBFLLJWZJMKMJDOZ9ECKWDOPWPYGV9PQIBTXOWPCTHPVGPVQUBBPSC9NVJLEZOITPJE9ZZNG9KOCMEZEHF9JZZSMPDRRAD9CSV9UVFXTZTTDOCWDYRIWMHAOYUO9SHRKW9MACEO9LPGZBSAWYQVKWMZDUNEE9ONRTDWTZEZVQEZXWHSPFKGAHBIKYBWGYCLZRZBDSVJIUXERPSF9UFQOZSLMMTGS9UCQASYAECTDZFWRBOAWMXOMQFQODFJQJGVOXHNJDLPWSGDDUPBXAOLFFUUYUAOVDMMTIEFTJWMFE9OPANAKKWIMXRHLNHMRZH9TCEONTVCSLADVJYZXWYHDMVBXFOCYFTOXHVZVCELZMUIJZHRALVHZ9NSVIK9VMBGRXX9GOUSGHSERBFIZGC9X9HVWV9VVGEWCPVQI9CFRGAYVPSVULWQNKTCJUZBYYVPBNHSTIMXVZOCUVRYJIRCT9LFANYATSAPPDFORPAYWXWFG9CECXRKTV9PMQSZMOVVYYKX9JBAVRYSCMXWFM9QVS9QUUPKNJSUSOMYCUIOOAAD9NHZZKOYMMZNSQYSDYBBYMRTEWQUYUHFLVHUUFWIXQRAWSPZPKETGKJSFGMAKFMVSTQEDLTUJYQONNBWVJDHTLVIGOSKKPQPDSHYUAAFAOVMOAXOMRWOBTAOIAGVBRTFELKIFZNFSADZYSGHBWLTOEJCUDFVRPPLWDMHSOJBWEBRTCRIEEMDSEKKXPRZGGIJYHOWYQKSJH9MFCHYKBGXUWYXQGJRXFLCDHVV9TKDITLCAIRZTJPACOPINMUC9RLFTYVALBRVA9OBMOYMFHQWC9TAISJODUMBIWC9RSPSVZGAJOZXTULEOPTKCGYYOOKDEOSKLDEGDXRQHIEMGZUXMOW999999999999999999999999999GD9999999999999999999999999JAYNICD99999999999B99999999YGOQEYWBEGZJGLJSPPFFVWA9RIDDZRFEGDZRZGATZTKSETYBXQHKQZGYLJVOCIQEDVCWAWBK9TUPK9DIWCEWQXNQCKQQEAAOFSEUGCGPMXCIBPCR9ASZEDDPRGWBLOVDSKMDCAUBYFLDTRJRUCAPCWSSSVILNZ9999UQJFWFRNIWMVPSICVZSWBW9JMEVVDHQRJWOPVHRBDMNEGRCVNYRLKPHZLQMLVYYBFVUXAIXYRRLW99999999999999999999999999999999USHXKUSRF999999999K99999999RC999999999PTA9999999999999";

    let input_243 = TryteBuf::try_from_str(input_243)
        .unwrap()
        .as_trits()
        .encode::<T5B1Buf>();
    let input_8019 = TryteBuf::try_from_str(input_8019)
        .unwrap()
        .as_trits()
        .encode::<T5B1Buf>();

    let mut group = c.benchmark_group("CurlP");
    group.throughput(Throughput::Elements(BATCH_SIZE as u64));
    for input in [input_243, input_8019].iter() {
        let length = input.len();

        // Using T5B1 directly.
        group.bench_with_input(
            BenchmarkId::new("Batched", format!("{} T5B1", length)),
            input,
            |b, i| b.iter(|| batched_hasher_t5b1(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("Unbatched", format!("{} T5B1", length)),
            input,
            |b, i| b.iter(|| unbatched_hasher_t5b1(i)),
        );

        // Encoding from T5B1 to T1B1 first.
        group.bench_with_input(
            BenchmarkId::new("Batched + Encode", format!("{} T5B1", length)),
            input,
            |b, i| {
                b.iter(|| {
                    batched_hasher_encoding_t5b1(i);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Unbatched + Encode", format!("{} T5B1", length)),
            input,
            |b, i| {
                b.iter(|| {
                    unbatched_hasher_encoding_t5b1(i);
                })
            },
        );

        // Using T1B1 directly.
        let input = &input.encode::<T1B1Buf>();
        group.bench_with_input(
            BenchmarkId::new("Batched", format!("{} T1B1", length)),
            input,
            |b, i| b.iter(|| batched_hasher_t1b1(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("Unbatched", format!("{} T1B1", length)),
            input,
            |b, i| b.iter(|| unbatched_hasher_t1b1(i)),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_hasher);
criterion_main!(benches);
