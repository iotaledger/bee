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

use bee_signing::ternary::wots::{normalize, NormalizeError};

use bee_ternary::{T1B1Buf, TryteBuf};

#[test]
fn invalid_message_length() {
    let hash = TryteBuf::try_from_str("CEFLDDLMF9TO9ZLLTYXINXPYZHEFTXKWKWZXEIUD")
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>();

    assert_eq!(
        normalize(&hash).err(),
        Some(NormalizeError::InvalidMessageLength(hash.len()))
    );
}

#[test]
fn input_output() {
    let tests = [
        (
            "YSQMIFUQFJNLFAPAETRWNWUX9LSTTCERCIOBDZIDHVRVNPQNHTSNWYKSRFDOCQGXFTJY9HIGNND9RBHYF",
            "MJQMIFUQFJNLFAPAETRWNWUX9LSMMMMMDIOBDZIDHVRVNPQNHTSNWYHSRFDOCQGXFTJY9HIGNND9RBHYF",
        ),
        (
            "FLMLSYHTEIXHEKZKABOVAZBEZNRAAM99KYXHR9IZZTF9DXNS9GNZDEZZACQTS9EPYNZYUFWFVQS9UOGFR",
            "NNHLSYHTEIXHEKZKABOVAZBEZNRTAM99KYXHR9IZZTF9DXNS9GNZDEMMMMMBS9EPYNZYUFWFVQS9UOGFR",
        ),
        (
            "U9NWKLJUSHCVUDVJRFMCIZHUDMPLBZFTPCTOMKVGTEIRDSTBFDAOYIGEWSAXEZUFXO9HMDGKRH9ZJEJSY",
            "NNNNFLJUSHCVUDVJRFMCIZHUDMPFBZFTPCTOMKVGTEIRDSTBFDAOYINNRSAXEZUFXO9HMDGKRH9ZJEJSY",
        ),
        (
            "TDA9LYGIE9OVLYGRAVHWYXPXNJMRZAMALYVJNRJP9SC9KYYSHIBHJVSOQIOKNYCNYYAPIUNXLDBWROWKN",
            "NZA9LYGIE9OVLYGRAVHWYXPXNJMNNUMALYVJNRJP9SC9KYYSHIBHJVMMMMBKNYCNYYAPIUNXLDBWROWKN",
        ),
        (
            "OYXALGQ9HQVKD9FPNDZXNUHT9WGHKVNXJRLTMYQFUKEITR9KPSIVGTBFC9DKSN9GDJJJYSPJYXISGPZCN",
            "MEXALGQ9HQVKD9FPNDZXNUHT9WGYKVNXJRLTMYQFUKEITR9KPSIVGTNEC9DKSN9GDJJJYSPJYXISGPZCN",
        ),
        (
            "INQE9N9JPQPS9JOEQGZGJYRSJBILNRLE9DAZVVVVFCZNAZERHWXXPTBUUIPZJYQGBPKYC9AEFMJN9RSAC",
            "MMUE9N9JPQPS9JOEQGZGJYRSJBIMMBLE9DAZVVVVFCZNAZERHWXXPTYUUIPZJYQGBPKYC9AEFMJN9RSAC",
        ),
        (
            "DAXOGMLCVIGJWBTMFLBZLRVD9ZLJUQLSJGJF9XAAGVKQSHTSTQXJAWOJROXDBOWUYIF9JASOCIXFPWTIR",
            "NNNNNTLCVIGJWBTMFLBZLRVD9ZLEUQLSJGJF9XAAGVKQSHTSTQXJAWMMHOXDBOWUYIF9JASOCIXFPWTIR",
        ),
        (
            "UVIQJKFZDPZVCNLTQWUNLWXSGFIOMD9DYHOMAJZDNW9ONSLRNZCBZAKNHLDJLHBIMCPNHRCCBWBSRSUBB",
            "LVIQJKFZDPZVCNLTQWUNLWXSGFIMME9DYHOMAJZDNW9ONSLRNZCBZANNZLDJLHBIMCPNHRCCBWBSRSUBB",
        ),
        (
            "WDVGEKTJYIHISJXHFLRFGTLPNUDTWBKTSKNLJXO9JUUHBOZAU9G9MLVZEDZILUIDYTPKCLDHPYNEJ9YJN",
            "NNNOEKTJYIHISJXHFLRFGTLPNUDNOBKTSKNLJXO9JUUHBOZAU9G9MLNNTDZILUIDYTPKCLDHPYNEJ9YJN",
        ),
        (
            "XHGMBGNQOLRRCPWRZTQJEWYOEMVISGVUXCTIWCFMXWNYBKFVXPUJOPWGQZQXTYNJZUQXCQTIZFXSXOTEX",
            "MMMMKGNQOLRRCPWRZTQJEWYOEMVMLGVUXCTIWCFMXWNYBKFVXPUJOPMMMMMYTYNJZUQXCQTIZFXSXOTEX",
        ),
    ];

    for test in tests.iter() {
        let input_trits = TryteBuf::try_from_str(test.0).unwrap().as_trits().encode::<T1B1Buf>();
        let output_trits = TryteBuf::try_from_str(test.1).unwrap().as_trits().encode::<T1B1Buf>();
        let normalized_trits = normalize(&input_trits).unwrap().encode::<T1B1Buf>();

        assert_eq!(output_trits, normalized_trits);
    }
}
