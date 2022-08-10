// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::*;

#[test]
fn conv() {
    let trits = TritBuf::<T3B1Buf>::from_trits(&[
        Btrit::PlusOne,
        Btrit::Zero,
        Btrit::NegOne,
        Btrit::Zero,
        Btrit::PlusOne,
        Btrit::Zero,
    ]);

    let s0 = trits
        .chunks(3)
        .map(|trits| {
            char::from(Tryte::from_trits([
                trits.get(0).unwrap(),
                trits.get(1).unwrap(),
                trits.get(2).unwrap(),
            ]))
        })
        .collect::<String>();

    assert_eq!(s0.as_str(), "SC");

    let s1 = trits.as_trytes().iter().map(|t| char::from(*t)).collect::<String>();

    assert_eq!(s1.as_str(), "SC");

    let s2 = trits.iter_trytes().map(char::from).collect::<String>();

    assert_eq!(s2.as_str(), "SC");
}
