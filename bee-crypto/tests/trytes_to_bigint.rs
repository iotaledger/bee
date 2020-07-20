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

use bee_crypto::ternary::bigint::{binary_representation::U8Repr, endianness::BigEndian, I384, T243};
use bee_ternary::{T1B1Buf, TritBuf, TryteBuf};

use std::convert::TryFrom;

#[test]
fn trytes_to_i384_be_u8_1() {
    const INPUT_TRYTES: &str = "EMIDYNHBWMBCXVDEFOFWINXTERALUKYYPPHKP9JJFGJEIUY9MUDVNFZHMMWZUYUSWAIOWEVTHNWMHANBH";

    const TRYTES_AS_I384_BE_U8: [u8; 48] = [
        236, 51, 87, 194, 177, 242, 107, 101, 103, 168, 5, 66, 166, 81, 89, 243, 253, 197, 196, 167, 255, 13, 7, 255,
        82, 193, 78, 211, 157, 243, 205, 238, 142, 59, 98, 37, 11, 4, 89, 43, 160, 190, 239, 144, 158, 28, 67, 19,
    ];

    let trytes = TryteBuf::try_from_str(INPUT_TRYTES);
    assert!(trytes.is_ok());
    let trytes = trytes.unwrap();
    let trit_buf: TritBuf<T1B1Buf> = trytes.as_trits().encode();
    let t243 = T243::new(trit_buf);
    let t242 = t243.into_t242();

    let converted_i384 = I384::<BigEndian, U8Repr>::try_from(t242);
    assert!(converted_i384.is_ok());
    let converted_i384 = converted_i384.unwrap();

    let expected_i384 = I384::<BigEndian, U8Repr>::from_array(TRYTES_AS_I384_BE_U8);
    assert_eq!(converted_i384, expected_i384);
}

#[test]
fn trytes_to_i384_be_u8_2() {
    const INPUT_TRYTES: &str = "DJ9WGAKRZOMH9KVRCHGCDCREXZVDKY9FXAXVSLELYADXHQCQQSMQYAEEBTEIWTQDUZIOFSFLBQQA9RUPX";

    const TRYTES_AS_I384_BE_U8: [u8; 48] = [
        184, 83, 213, 85, 177, 195, 33, 31, 86, 245, 168, 205, 110, 156, 207, 177, 122, 174, 237, 75, 210, 56, 85, 12,
        191, 10, 209, 77, 84, 232, 148, 185, 210, 97, 59, 96, 214, 31, 247, 230, 30, 67, 122, 93, 101, 171, 72, 105,
    ];

    let trytes = TryteBuf::try_from_str(INPUT_TRYTES);
    assert!(trytes.is_ok());
    let trytes = trytes.unwrap();
    let trit_buf: TritBuf<T1B1Buf> = trytes.as_trits().encode();
    let t243 = T243::new(trit_buf);
    let t242 = t243.into_t242();

    let converted_i384 = I384::<BigEndian, U8Repr>::try_from(t242);
    assert!(converted_i384.is_ok());
    let converted_i384 = converted_i384.unwrap();

    let expected_i384 = I384::<BigEndian, U8Repr>::from_array(TRYTES_AS_I384_BE_U8);
    assert_eq!(converted_i384, expected_i384);
}
