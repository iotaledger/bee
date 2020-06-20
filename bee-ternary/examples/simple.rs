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

// TRYTE STRING INTO TRITS
//
// Run with:
//
// ```
// cargo run --example simple
// ```
//

use bee_ternary::{T1B1Buf, TryteBuf};

fn main() {
    // String with Trytes [alphabet: A-Z and 9]
    const HELLO__TRYTES_STRING: &str = "HELLOWORLD9";

    // Convert Trytes string to trits
    let hello_trits = TryteBuf::try_from_str(HELLO__TRYTES_STRING)
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>();

    // Print trits
    println!("'HELLOWORLD9' in trits: {}", hello_trits);
}
