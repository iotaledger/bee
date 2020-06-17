
//! TRYTE STRING INTO TRITS
//!
//! Run with:
//!
//! ```
//! cargo run --example simple
//! ```
//! 


use bee_ternary::{T1B1Buf, TryteBuf};


fn main() {
    // String with Trytes [alphabet: A-Z and 9]
    const HELLO__TRYTES_STRING: &str = "HELLOWORLD9";

    // Convert Trytesstring to trits
    let hello_trits = TryteBuf::try_from_str(HELLO__TRYTES_STRING)
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>();

    // Print trits
    println!("'HELLOWORLD9' in trits: {}", hello_trits);

}