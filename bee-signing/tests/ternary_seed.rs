// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_crypto::ternary::sponge::{Kerl, Sponge};
use bee_signing::ternary::seed::{Error, Seed};
use bee_ternary::{T1B1Buf, TritBuf, TryteBuf};

use std::str::FromStr;

const SEED: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9ABCDEFGHIJKLMNOPQRSTUVWXYZ9";

fn subseed_generic<S: Sponge + Default>(seed_string: &str, subseed_strings: &[&str]) {
    let seed = Seed::from_str(seed_string).unwrap();

    for (i, subseed_string) in subseed_strings.iter().enumerate() {
        let subseed = seed.subseed(i);
        let subseed_trits = TryteBuf::try_from_str(subseed_string)
            .unwrap()
            .as_trits()
            .encode::<T1B1Buf>();

        assert_eq!(subseed.as_trits(), subseed_trits.as_slice());
    }
}

#[test]
fn subseed_kerl() {
    subseed_generic::<Kerl>(
        SEED,
        &[
            "APSNZAPLANAGSXGZMZYCSXROJ9KUX9HVOPODQHMWNJOCGBKRIOOQKYGPFAIQBYNIODMIWMFKJGKRWFFPY",
            "PXQMW9VMXGYTEPYPIASGPQ9CAQUQWNSUIIVHFIEAB9C9DHNNCWSNJKSBEAKYIBCYOZDDTQANEKPGJPVIY",
            "ZUJWIFUVFGOGDNMTFDVZGTWVCBVIK9XQQDQEKJSKBXNGLFLLIPTVUHHPCPKNMBFMATPYJVOH9QTEVOYTW",
            "OCHUZGFIX9VXXMBJXPKAPZHXIOCLAEKREMCKQIYQPXQQLRTOEUQRCZIYVSLUTJQGISGDRDSCERBOEEI9C",
            "GWTMVQWHHCYFXVHGUYYZHUNXICJLMSOZVBAZOIZIWGBRAXMFDUBLP9NVIFEFFRARYIHNGPEBLNUECABKW",
            "XWIYCHCVZEXOPXCQEJUGPMGVAIYBULVHWDD9YWMAZNJQEISHOBMYFHZKCBT9GWCSRQSFURKF9I9ITWEUC",
            "XRBHXHE9IVEDFHQPNNMYOPXOLPXRBSYCGQNMRFKYENRJZLZAVMFLUCWWCNBFPKOSHF9UPMFFEWAWAHJP9",
            "IP9DGBVAPNHHDP9CXOBYRLTYVJCQYUUWNWGNFUSDRKFIIAVPYPQDASDULPJBBEBOQATDHV9PVXYIJFQTA",
            "XSGWTBAECBMTKEHXNYAVSYRPLASPJSHPIWROHRLDFUEKISEMCMXYGRZMPZCEAKZ9UKQBA9LEQFXWEMZPD",
            "JXCAHDZVVCMGIGWJFFVDRFCHKBVAWTSLWIPZYGBECFXJQPDNDYJTEYCBHSRPDMPFEPWZUMDEIPIBW9SI9",
        ],
    );
}

#[test]
fn from_str_invalid_length() {
    let trytes = "VBAZOIZIWGBRAXMFDUBLP";

    assert_eq!(
        Seed::from_str(&trytes).err(),
        Some(Error::InvalidLength(trytes.len() * 3))
    );
}

#[test]
fn from_str_invalid_trytes() {
    let trytes = "APSNZAPL@NAGSXGZMZYCSXROJ9KUX9HVOPODQHMWNJOCGBKRIOOQKYGPFAIQBYNIODMIWMFKJGKRWFFPY";

    assert_eq!(Seed::from_str(&trytes).err(), Some(Error::InvalidTrytes));
}

#[test]
fn from_trits_invalid_length() {
    let trits = TritBuf::zeros(42);

    assert_eq!(
        Seed::from_trits(trits.clone()).err(),
        Some(Error::InvalidLength(trits.len()))
    );
}

#[test]
fn to_trits_from_trits() {
    for _ in 0..10 {
        let seed_1 = Seed::rand();
        let seed_2 = Seed::from_trits(seed_1.as_trits().to_buf()).unwrap();

        assert_eq!(seed_1.as_trits(), seed_2.as_trits());
    }
}
