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
